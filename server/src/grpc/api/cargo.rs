//! Cargo-related handlers

use crate::grpc::client::GrpcClients;
use crate::grpc::server::{CargoConfirmationRequest, CargoConfirmationResponse};
use geo_types::{Coord, LineString};
use lib_common::time::{DateTime, Duration, Utc};
use polyline;
use postmark::api::email::*;
use postmark::reqwest::PostmarkClient;
use postmark::*;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use svc_storage_client_grpc::prelude::{flight_plan, vertiport};
use svc_storage_client_grpc::prelude::{AdvancedSearchFilter, Id};
use svc_storage_client_grpc::simple_service::Client as _;
use svc_storage_client_grpc::simple_service_linked::Client;
use tokio::sync::OnceCell;
use tonic::Status;

/// Postmark token
pub static POSTMARK_TOKEN: OnceCell<String> = OnceCell::const_new();

// TODO(R5): no-reply@aetheric.nl
/// Aetheric's email address
const AETHERIC_EMAIL_ADDRESS: &str = "info@aetheric.nl";

#[derive(Debug)]
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
    path: Vec<Coord>,
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

#[derive(Debug, PartialEq)]
enum FlightPlanError {
    Data,
    Path,
    OriginVertiportId,
    TargetVertiportId,
    OriginTimeslotStart,
    TargetTimeslotEnd,
}

impl Display for FlightPlanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FlightPlanError::Data => write!(f, "Data not found"),
            FlightPlanError::Path => write!(f, "Path not found"),
            FlightPlanError::OriginVertiportId => write!(f, "Origin vertiport ID not found"),
            FlightPlanError::TargetVertiportId => write!(f, "Target vertiport ID not found"),
            FlightPlanError::OriginTimeslotStart => write!(f, "Origin timeslot start not found"),
            FlightPlanError::TargetTimeslotEnd => write!(f, "Target timeslot end not found"),
        }
    }
}

impl TryFrom<flight_plan::Object> for PlanData {
    type Error = FlightPlanError;

    fn try_from(object: flight_plan::Object) -> Result<PlanData, Self::Error> {
        let id = object.id;
        let data = object.data.ok_or(FlightPlanError::Data)?;

        let path: Vec<Coord> = data
            .path
            .ok_or_else(|| {
                grpc_error!("Could not get path.");
                FlightPlanError::Path
            })?
            .points
            .into_iter()
            .map(|p| Coord { x: p.x, y: p.y })
            .collect();

        if path.len() < 2 {
            grpc_error!("Path has less than 2 points.");
            return Err(FlightPlanError::Path);
        }

        let origin = path[0];
        let (origin_latitude, origin_longitude) = (origin.y, origin.x);
        let target = path[path.len() - 1];
        let (target_latitude, target_longitude) = (target.y, target.x);

        let origin_vertiport_id = data
            .origin_vertiport_id
            .ok_or(FlightPlanError::OriginVertiportId)?;

        let target_vertiport_id = data
            .target_vertiport_id
            .ok_or(FlightPlanError::TargetVertiportId)?;

        let origin_timeslot_start: DateTime<Utc> = data
            .origin_timeslot_start
            .ok_or(FlightPlanError::OriginTimeslotStart)?
            .into();

        let target_timeslot_end: DateTime<Utc> = data
            .target_timeslot_end
            .ok_or(FlightPlanError::TargetTimeslotEnd)?
            .into();

        Ok(PlanData {
            id,
            origin_latitude,
            origin_longitude,
            target_latitude,
            target_longitude,
            origin_vertiport_id,
            target_vertiport_id,
            origin_timeslot_start,
            target_timeslot_end,
            path,
        })
    }
}

impl TryFrom<vertiport::Object> for VertiportData {
    type Error = Status;

    fn try_from(object: vertiport::Object) -> Result<Self, Self::Error> {
        object
            .data
            .map(|data| VertiportData {
                name: data.name,
                address: data.description,
            })
            .ok_or_else(|| Status::internal("Vertiport data not found"))
    }
}

#[cfg(not(tarpaulin_include))]
// no_coverage: (Rnever) not unit testable, only integration tests
async fn get_plan_data(clients: &GrpcClients, flight_plan_id: &str) -> Result<PlanData, Status> {
    clients
        .storage
        .flight_plan
        .get_by_id(Id {
            id: flight_plan_id.to_string(),
        })
        .await
        .map_err(|e| Status::internal(format!("Could not get flight plan: {}", e)))?
        .into_inner()
        .try_into()
        .map_err(|e| Status::internal(format!("Could not get flight plan: {:?}", e)))
}

#[cfg(not(tarpaulin_include))]
// no_coverage: (Rnever) not unit testable, only integration tests
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

    let path: Vec<Coord> = flight_plans.into_iter().flat_map(|f| f.path).collect();
    let polyline = polyline::encode_coordinates(LineString::new(path), 5).unwrap_or("".to_string());

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

#[cfg(not(tarpaulin_include))]
// no_coverage: (Rnever) not unit testable, only integration tests
async fn get_vertiport_data(
    clients: &GrpcClients,
    vertiport_id: &str,
) -> Result<VertiportData, Status> {
    clients
        .storage
        .vertiport
        .get_by_id(Id {
            id: vertiport_id.to_string(),
        })
        .await
        .map_err(|e| Status::internal(format!("Could not get vertiport: {}", e)))?
        .into_inner()
        .try_into()
}

#[cfg(not(tarpaulin_include))]
// no_coverage: (Rnever) not unit testable, only integration tests
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
        .unwrap_or("there") // hello there!
        .to_string();

    let email = if user_data.email.is_empty() {
        AETHERIC_EMAIL_ADDRESS.to_string()
    } else {
        user_data.email
    };

    Ok(UserData { name, email })
}

/// Sends a confirmation email to the user
#[cfg(not(tarpaulin_include))]
// no_coverage: (Rnever) not unit testable, only integration tests
pub async fn cargo_confirmation(
    request: CargoConfirmationRequest,
) -> Result<CargoConfirmationResponse, Status> {
    grpc_info!("entry.");

    let padding = Duration::try_minutes(10)
        .ok_or_else(|| Status::internal("Could not create time padding"))?;

    let dt_format = "%Y-%m-%d %H:%M UTC%z";

    let clients = crate::grpc::client::get_clients().await;

    let parcel_data = get_parcel_data(clients, &request.parcel_id).await?;

    let origin_vertiport_data =
        get_vertiport_data(clients, &parcel_data.origin_vertiport_id).await?;

    let target_vertiport_data =
        get_vertiport_data(clients, &parcel_data.target_vertiport_id).await?;

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

    let user_data = get_user_data(clients, &user_id).await?;
    let postmark_token = POSTMARK_TOKEN
        .get()
        .ok_or_else(|| Status::internal("Postmark token not found"))?;

    // TODO(R5): Get these from svc-cargo. Not needed for demo.
    let flight_price = 0.0;
    let network_fee = 0.0;
    let tax = 0.0;
    let details = vec![
        Details {
            description: "Network Fee".to_string(),
            amount: format!("{:.2}", network_fee),
        },
        Details {
            description: "VAT".to_string(),
            amount: format!("{:.2}", tax),
        },
    ];

    let total_price = format!("{:.2}", flight_price + network_fee + tax);
    let flight_price = format!("{:.2}", flight_price);
    let currency = "EUR".to_string();
    let invoice_date = Utc::now().format(dt_format).to_string();
    // TODO(R5): no actual payments in demo
    let invoice_id = rand::random::<u16>().to_string();

    let client = PostmarkClient::builder()
        .base_url(POSTMARK_API_URL)
        .token(postmark_token.to_string())
        .build();

    let mut model = TemplateModel::default();
    model.insert("customer_name", user_data.name);
    model.insert("customer_dropoff_time", dropoff_time);
    model.insert("customer_pickup_time", pickup_time);
    model.insert("parcel_weight_kg", format!("{:.2}", parcel_data.weight_kg));
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

    let response = SendEmailWithTemplateRequest::builder()
        .from(AETHERIC_EMAIL_ADDRESS)
        .to(user_data.email)
        .template_model(model)
        .template_alias("demo-confirmation")
        .build()
        .execute(&client)
        .await
        .map_err(|e| Status::internal(format!("Could not send email: {}", e)))?;

    let success = response.error_code == 0;
    if !success {
        grpc_error!("Could not send email: {:?}", response);
        return Err(Status::internal("Could not send email"));
    }

    grpc_info!("success={}.", success);
    Ok(CargoConfirmationResponse { success })
}

#[cfg(test)]
mod tests {
    use super::*;
    use svc_storage_client_grpc::prelude::{GeoLineStringZ, GeoPointZ};

    #[test]
    fn test_try_from_flight_plan_object() {
        let data = flight_plan::Data {
            origin_vertiport_id: Some("origin".to_string()),
            target_vertiport_id: Some("target".to_string()),
            ..flight_plan::mock::get_data_obj()
        };

        let mut object = flight_plan::Object {
            id: "test".to_string(),
            data: None,
        };
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::Data);

        let tmp = data.clone();
        object.data = Some(tmp);
        PlanData::try_from(object.clone()).unwrap();

        let mut tmp = data.clone();
        tmp.path = None;
        object.data = Some(tmp);
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::Path);

        let mut tmp = data.clone();
        tmp.path = Some(GeoLineStringZ { points: vec![] }); // empty path
        object.data = Some(tmp);
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::Path);

        let mut tmp = data.clone();
        tmp.path = Some(GeoLineStringZ {
            points: vec![GeoPointZ {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }],
        }); // only one point
        object.data = Some(tmp);
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::Path);

        let mut tmp = data.clone();
        tmp.origin_vertiport_id = None;
        object.data = Some(tmp);
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::OriginVertiportId);

        let mut tmp = data.clone();
        tmp.target_vertiport_id = None;
        object.data = Some(tmp);
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::TargetVertiportId);

        let mut tmp = data.clone();
        tmp.origin_timeslot_start = None;
        object.data = Some(tmp);
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::OriginTimeslotStart);

        let mut tmp = data.clone();
        tmp.target_timeslot_end = None;
        object.data = Some(tmp);
        let error = PlanData::try_from(object.clone()).unwrap_err();
        assert_eq!(error, FlightPlanError::TargetTimeslotEnd);
    }

    #[test]
    fn test_flight_plan_error_display() {
        assert_eq!(FlightPlanError::Data.to_string(), "Data not found");
        assert_eq!(FlightPlanError::Path.to_string(), "Path not found");
        assert_eq!(
            FlightPlanError::OriginVertiportId.to_string(),
            "Origin vertiport ID not found"
        );
        assert_eq!(
            FlightPlanError::TargetVertiportId.to_string(),
            "Target vertiport ID not found"
        );
        assert_eq!(
            FlightPlanError::OriginTimeslotStart.to_string(),
            "Origin timeslot start not found"
        );
        assert_eq!(
            FlightPlanError::TargetTimeslotEnd.to_string(),
            "Target timeslot end not found"
        );
    }
}
