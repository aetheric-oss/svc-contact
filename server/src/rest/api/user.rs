//! Rest API implementations of user-related operations
/// openapi generated rest types
pub use super::rest_types::*;
use crate::grpc::client::GrpcClients;
use axum::{extract::Extension, Json};
use hyper::StatusCode;

use svc_storage_client_grpc::prelude::*;

impl From<SignupRequest> for user::Data {
    fn from(req: SignupRequest) -> Self {
        user::Data {
            // TODO(R5): Update this with the right auth method
            auth_method: user::AuthMethod::Local.into(),
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
pub async fn signup(
    Extension(grpc_clients): Extension<GrpcClients>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<String>, StatusCode> {
    rest_debug!("entry.");

    let data: user::Data = payload.clone().into();
    let user_id = grpc_clients
        .storage
        .user
        .insert(data)
        .await
        .map_err(|e| {
            rest_debug!("failed to insert data from payload: {:#?}", payload);
            rest_error!("failed to insert user: {}.", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner()
        .object
        .ok_or_else(|| {
            rest_error!("failed to insert user: no user object returned.");
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
    async fn test_signup_success() {
        lib_common::logger::get_log_handle().await;
        ut_info!("Start.");

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
