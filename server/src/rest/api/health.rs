//! Rest API implementations for health checks
/// openapi generated rest types
pub use super::rest_types::*;
use crate::grpc::client::GrpcClients;
use axum::extract::Extension;
use hyper::StatusCode;

use svc_storage_client_grpc::prelude::*;

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
    rest_debug!("entry.");

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
        rest_error!("{}.", &error_msg);
        ok = false;
    }

    match ok {
        true => {
            rest_debug!("healthy, all dependencies running.");
            Ok(())
        }
        false => {
            rest_error!("unhealthy, 1+ dependencies down.");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

impl From<SignupRequest> for user::Data {
    fn from(req: SignupRequest) -> Self {
        user::Data {
            auth_method: user::AuthMethod::Local.into(), // TODO(R5): Update this with the right auth method
            display_name: req.display_name,
            email: req.email,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_common::uuid::to_uuid;

    #[tokio::test]
    async fn test_health_check_success() {
        lib_common::logger::get_log_handle().await;
        ut_info!("Start.");

        // Mock the GrpcClients extension
        let config = crate::Config::default();
        let grpc_clients = GrpcClients::default(config); // Replace with your own mock implementation

        // Call the health_check function
        let result = health_check(Extension(grpc_clients)).await;

        // Assert the expected result
        println!("{:?}", result);
        assert!(result.is_ok());

        ut_info!("Success.");
    }
}
