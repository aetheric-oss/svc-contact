//! Cargo-related handlers

use crate::grpc::client::GrpcClients;
use crate::grpc::server::{CargoConfirmationRequest, CargoConfirmationResponse};
use chrono::{DateTime, Duration, Utc};
use polyline;
use postmark::api::email::*;
use postmark::reqwest::PostmarkClient;
use postmark::*;
use serde::Serialize;
use svc_storage_client_grpc::prelude::{AdvancedSearchFilter, Id};
use svc_storage_client_grpc::simple_service::Client as _;
use svc_storage_client_grpc::simple_service_linked::Client;
use tokio::sync::OnceCell;
use tonic::Status;

/// Postmark token
pub static POSTMARK_TOKEN: OnceCell<String> = OnceCell::const_new();

/// Aetheric's email address
const AETHERIC_EMAIL_ADDRESS: &str = "info@aetheric.nl"; // TODO(R5): no-reply@aetheric.nl

struct PlanData {
    id: String,
    origin_latitude: f64,
    origin_longitude: f64,
    target_latitude: f64,
    target_longitude: f64,
    origin_vertiport_id: String,
    target_vertiport_id: String,
    origin_timeslot_start: DateTime<Utc>,
    target_timeslot_end: DateTime<Utc>,
    path: Vec<geo_types::Coord>,
}

/// Parcel information needed for a confirmation email
struct ParcelData {
    /// Weight in kilograms
    weight_kg: f32,

    /// pickup flight plan
    origin_vertiport_id: String,

    /// Dropoff flight plan
    target_vertiport_id: String,

    /// origin timeslot start
    origin_timeslot_start: DateTime<Utc>,

    /// target timeslot end
    target_timeslot_end: DateTime<Utc>,

    /// origin latitude
    origin_latitude: f64,

    /// origin longitude
    origin_longitude: f64,

    /// target latitude
    target_latitude: f64,

    /// target longitude
    target_longitude: f64,

    /// Full path
    polyline: String,
}

struct VertiportData {
    name: String,
    address: String,
}

struct UserData {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct Details {
    amount: String,
    description: String,
}

async fn get_plan_data(clients: &GrpcClients, flight_plan_id: &str) -> Result<PlanData, Status> {
    let response = clients
        .storage
        .flight_plan
        .get_by_id(Id {
            id: flight_plan_id.to_string(),
        })
        .await
        .map_err(|e| Status::internal(format!("Could not get flight plan: {}", e)))?
        .into_inner();

    let id = response.id;
    let data = response
        .data
        .ok_or_else(|| Status::internal("Flight plan data not found"))?;

    let path = data
        .path
        .ok_or_else(|| Status::internal("Path not found"))?
        .points;

    let (origin_latitude, origin_longitude) = path
        .first()
        .map(|p| (p.latitude, p.longitude))
        .ok_or_else(|| Status::internal("Origin path not found"))?;

    let (target_latitude, target_longitude) = path
        .last()
        .map(|p| (p.latitude, p.longitude))
        .ok_or_else(|| Status::internal("Target path not found"))?;

    Ok(PlanData {
        id,
        origin_latitude,
        origin_longitude,
        target_latitude,
        target_longitude,
        origin_vertiport_id: data
            .origin_vertiport_id
            .ok_or_else(|| Status::internal("Origin vertiport ID not found"))?,
        target_vertiport_id: data
            .target_vertiport_id
            .ok_or_else(|| Status::internal("Target vertiport ID not found"))?,
        origin_timeslot_start: data
            .origin_timeslot_start
            .ok_or_else(|| Status::internal("Origin timeslot start not found"))?
            .into(),
        target_timeslot_end: data
            .target_timeslot_end
            .ok_or_else(|| Status::internal("Target timeslot end not found"))?
            .into(),
        path: path
            .into_iter()
            .map(|p| geo_types::Coord {
                x: p.longitude,
                y: p.latitude,
            })
            .collect(),
    })
}

async fn get_parcel_data(clients: &GrpcClients, parcel_id: &str) -> Result<ParcelData, Status> {
    let parcel_data = clients
        .storage
        .parcel
        .get_by_id(Id {
            id: parcel_id.to_string(),
        })
        .await
        .map_err(|e| Status::internal(format!("Could not get parcel: {}", e)))?
        .into_inner()
        .data
        .ok_or_else(|| Status::internal("Parcel data not found"))?;

    let filter =
        AdvancedSearchFilter::search_equals("parcel_id".to_string(), parcel_id.to_string());

    let mut origin_flight_id: Option<String> = None;
    let mut target_flight_id: Option<String> = None;
    let mut flight_plan_ids: Vec<String> = vec![];
    clients
        .storage
        .flight_plan_parcel
        .search(filter)
        .await
        .map_err(|e| Status::internal(format!("Could not get flight plan: {}", e)))?
        .into_inner()
        .list
        .into_iter()
        .for_each(|f| {
            if f.deliver {
                target_flight_id = Some(f.flight_plan_id.clone());
            }

            if f.acquire {
                origin_flight_id = Some(f.flight_plan_id.clone());
            }

            flight_plan_ids.push(f.flight_plan_id);
        });

    let origin_flight_id =
        origin_flight_id.ok_or_else(|| Status::internal("Origin flight ID not found"))?;
    let target_flight_id =
        target_flight_id.ok_or_else(|| Status::internal("Target flight ID not found"))?;

    let mut flight_plans = vec![];
    for id in flight_plan_ids.iter() {
        let flight_data = get_plan_data(clients, id)
            .await
            .map_err(|e| Status::internal(format!("Could not get flight plan: {}", e)))?;
        flight_plans.push(flight_data);
    }

    flight_plans.sort_by(|a, b| a.origin_timeslot_start.cmp(&b.origin_timeslot_start));

    let (origin_vertiport_id, origin_timeslot_start, origin_latitude, origin_longitude) =
        flight_plans
            .iter()
            .find(|fp| fp.id == origin_flight_id)
            .map(|fp| {
                (
                    fp.origin_vertiport_id.clone(),
                    fp.origin_timeslot_start,
                    fp.origin_latitude,
                    fp.origin_longitude,
                )
            })
            .ok_or_else(|| Status::internal("Origin flight plan not found"))?;

    let (target_vertiport_id, target_timeslot_end, target_latitude, target_longitude) =
        flight_plans
            .iter()
            .find(|fp| fp.id == target_flight_id)
            .map(|fp| {
                (
                    fp.target_vertiport_id.clone(),
                    fp.target_timeslot_end,
                    fp.target_latitude,
                    fp.target_longitude,
                )
            })
            .ok_or_else(|| Status::internal("Target flight plan not found"))?;

    let path: Vec<geo_types::Coord> = flight_plans.into_iter().flat_map(|f| f.path).collect();
    let polyline =
        polyline::encode_coordinates(geo_types::LineString::new(path), 5).unwrap_or("".to_string());

    let parcel = ParcelData {
        weight_kg: (parcel_data.weight_grams as f32) / 1000.0,
        origin_vertiport_id,
        target_vertiport_id,
        origin_timeslot_start,
        target_timeslot_end,
        origin_latitude,
        origin_longitude,
        target_latitude,
        target_longitude,
        polyline,
    };

    Ok(parcel)
}

async fn get_vertiport_data(
    clients: &GrpcClients,
    vertiport_id: &str,
) -> Result<VertiportData, Status> {
    let vertiport_data = clients
        .storage
        .vertiport
        .get_by_id(Id {
            id: vertiport_id.to_string(),
        })
        .await
        .map_err(|e| Status::internal(format!("Could not get vertiport: {}", e)))?
        .into_inner()
        .data
        .ok_or_else(|| Status::internal("Vertiport data not found"))?;

    Ok(VertiportData {
        name: vertiport_data.name,
        address: vertiport_data.description,
    })
}

async fn get_user_data(clients: &GrpcClients, user_id: &str) -> Result<UserData, Status> {
    let user_data = clients
        .storage
        .user
        .get_by_id(Id {
            id: user_id.to_string(),
        })
        .await
        .map_err(|e| Status::internal(format!("Could not get user: {}", e)))?
        .into_inner()
        .data
        .ok_or_else(|| Status::internal("User data not found"))?;

    let name = user_data
        .display_name
        .split_whitespace()
        .next()
        .unwrap_or("Customer")
        .to_string();
    let email = if user_data.email.is_empty() {
        AETHERIC_EMAIL_ADDRESS.to_string()
    } else {
        user_data.email
    };

    Ok(UserData { name, email })
}

/// Sends a confirmation email via postmark API
pub async fn cargo_confirmation(
    request: CargoConfirmationRequest,
) -> Result<CargoConfirmationResponse, Status> {
    let padding = Duration::try_minutes(10)
        .ok_or_else(|| Status::internal("Could not create time padding"))?;
    let dt_format = "%Y-%m-%d %H:%M %z";

    let clients = crate::grpc::client::get_clients().await;
    let parcel_data = get_parcel_data(&clients, &request.parcel_id).await?;
    let origin_vertiport_data =
        get_vertiport_data(&clients, &parcel_data.origin_vertiport_id).await?;
    let target_vertiport_data =
        get_vertiport_data(&clients, &parcel_data.target_vertiport_id).await?;
    let dropoff_time = (parcel_data.target_timeslot_end - padding)
        .format(dt_format)
        .to_string();
    let pickup_time = (parcel_data.origin_timeslot_start + padding)
        .format(dt_format)
        .to_string();

    let user_id = clients
        .storage
        .itinerary
        .get_by_id(Id {
            id: request.itinerary_id,
        })
        .await
        .map_err(|e| Status::internal(format!("Could not get itinerary: {}", e)))?
        .into_inner()
        .data
        .ok_or_else(|| Status::internal("Itinerary data not found"))?
        .user_id;

    let user_data = get_user_data(&clients, &user_id).await?;
    let postmark_token = POSTMARK_TOKEN
        .get()
        .ok_or_else(|| Status::internal("Postmark token not found"))?;

    // TODO(R5): Get these from svc-cargo. Not needed for demo.
    let flight_price = 0.0;
    let tax = 0.0;
    let details = vec![Details {
        description: "VAT".to_string(),
        amount: format!("{:.2}", tax),
    }];

    let total_price = flight_price + tax;
    let currency = "EUR".to_string();
    let invoice_date = Utc::now().format(dt_format).to_string();
    let invoice_id = rand::random::<u16>().to_string(); // TODO(R5): no actual payments in demo

    let client = PostmarkClient::builder()
        .base_url("https://api.postmarkapp.com/")
        .token(postmark_token.to_string())
        .build();

    let mut model = TemplateModel::default();
    model.insert("flight_price", total_price);
    model.insert("customer_name", user_data.name);
    model.insert("customer_dropoff_time", dropoff_time);
    model.insert("customer_pickup_time", pickup_time);
    model.insert("parcel_weight_kg", parcel_data.weight_kg);
    model.insert("origin_vertiport_name", origin_vertiport_data.name);
    model.insert("origin_vertiport_address", origin_vertiport_data.address);
    model.insert("target_vertiport_name", target_vertiport_data.name);
    model.insert("target_vertiport_address", target_vertiport_data.address);
    model.insert("origin_latitude", parcel_data.origin_latitude);
    model.insert("origin_longitude", parcel_data.origin_longitude);
    model.insert("target_latitude", parcel_data.target_latitude);
    model.insert("target_longitude", parcel_data.target_longitude);
    model.insert("encoded_polyline", parcel_data.polyline);
    model.insert("invoice_id", invoice_id);
    model.insert("invoice_date", invoice_date);
    model.insert("flight_price", flight_price);
    model.insert("receipt_add_details", details);
    model.insert("currency", currency);
    model.insert("total_price", total_price);

    let request = SendEmailWithTemplateRequest::builder()
        .from(AETHERIC_EMAIL_ADDRESS)
        .to(user_data.email)
        .template_model(model)
        .template_alias("demo-confirmation")
        .build();

    let response = request
        .execute(&client)
        .await
        .map_err(|e| Status::internal(format!("Could not send email: {}", e)))?;
    Ok(CargoConfirmationResponse {
        success: response.error_code == 0,
    })
}
