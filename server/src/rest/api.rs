//! Rest API implementations
/// openapi generated rest types
pub mod rest_types {
    include!("../../../openapi/types.rs");
}

pub use rest_types::*;

use crate::grpc::client::GrpcClients;
use axum::{extract::Extension, Json};
use hyper::StatusCode;

use svc_storage_client_grpc::prelude::{user::AuthMethod, *};

/// Provides a way to tell a caller if the service is healthy.
/// Checks dependencies, making sure all connections can be made.
#[utoipa::path(
    get,
    path = "/health",
    tag = "svc-contact",
    responses(
        (status = 200, description = "Service is healthy, all dependencies running."),
        (status = 503, description = "Service is unhealthy, one or more dependencies unavailable.")
    )
)]
#[cfg(not(tarpaulin_include))] // no way to make this fail with stubs
pub async fn health_check(
    Extension(grpc_clients): Extension<GrpcClients>,
) -> Result<(), StatusCode> {
    rest_debug!("(health_check) entry.");

    let mut ok = true;

    // FIXME - update/ uncomment this with the right dependencies.
    // This health check is to verify that ALL dependencies of this
    // microservice are running.
    if grpc_clients
        .storage
        .user
        .is_ready(ReadyRequest {})
        .await
        .is_err()
    {
        let error_msg = "svc-storage user unavailable.".to_string();
        rest_error!("(health_check) {}.", &error_msg);
        ok = false;
    }

    match ok {
        true => {
            rest_debug!("(health_check) healthy, all dependencies running.");
            Ok(())
        }
        false => {
            rest_error!("(health_check) unhealthy, 1+ dependencies down.");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

impl From<SignupRequest> for user::Data {
    fn from(req: SignupRequest) -> Self {
        user::Data {
            auth_method: AuthMethod::Local.into(), // TODO(R5): Update this with the right auth method
            display_name: req.display_name,
            email: req.email,
        }
    }
}

/// Example REST API function
#[utoipa::path(
    post,
    path = "/contact/signup",
    tag = "svc-contact",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "Request successful.", body = String),
        (status = 500, description = "Request unsuccessful."),
    )
)]
#[cfg(not(tarpaulin_include))] // no way to make this fail with stubs
pub async fn signup(
    Extension(grpc_clients): Extension<GrpcClients>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<String>, StatusCode> {
    rest_debug!("(signup) entry.");

    let data: user::Data = payload.into();
    let user_id = grpc_clients
        .storage
        .user
        .insert(data)
        .await
        .map_err(|e| {
            rest_error!("(signup) failed to insert user: {}.", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner()
        .object
        .ok_or_else(|| {
            rest_error!("(signup) failed to insert user: no user object returned.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .id;

    Ok(Json(user_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_common::uuid::to_uuid;

    #[tokio::test]
    async fn test_health_check_success() {
        lib_common::logger::get_log_handle().await;
        ut_info!("(test_health_check_success) Start.");

        // Mock the GrpcClients extension
        let config = crate::Config::default();
        let grpc_clients = GrpcClients::default(config); // Replace with your own mock implementation

        // Call the health_check function
        let result = health_check(Extension(grpc_clients)).await;

        // Assert the expected result
        println!("{:?}", result);
        assert!(result.is_ok());

        ut_info!("(test_health_check_success) Success.");
    }

    #[tokio::test]
    async fn test_signup_success() {
        lib_common::logger::get_log_handle().await;
        ut_info!("(test_signup_success) Start.");

        // Mock the GrpcClients extension
        let config = crate::Config::default();
        let grpc_clients = GrpcClients::default(config); // Replace with your own mock implementation

        // Mock the payload
        let payload = SignupRequest {
            display_name: "test".to_string(),
            email: "test@aetheric.nl".to_string(),
        };

        let id = signup(Extension(grpc_clients), Json(payload))
            .await
            .unwrap()
            .0;

        // check UUID format
        to_uuid(&id).unwrap();
    }
}
