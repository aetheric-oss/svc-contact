//! REST
//! provides server implementations for REST API

#[macro_use]
pub mod macros;
pub mod api;
pub mod server;

use std::fmt::{self, Display, Formatter};
use utoipa::OpenApi;

/// OpenAPI 3.0 specification for this service
#[derive(OpenApi, Copy, Clone, Debug)]
#[openapi(
    paths(
        api::health::health_check,
        api::user::signup
    ),
    components(
        schemas(
            api::rest_types::SignupRequest
        )
    ),
    tags(
        (name = "svc-contact", description = "svc-contact REST API")
    )
)]
#[cfg(not(tarpaulin_include))]
pub struct ApiDoc;

/// Errors with OpenAPI generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenApiError {
    /// Failed to export as JSON string
    Json,

    /// Failed to write to file
    FileWrite,
}

impl std::error::Error for OpenApiError {}

#[cfg(not(tarpaulin_include))] // Exclude from coverage, not possible to make this fail to generate JSON
impl Display for OpenApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OpenApiError::Json => write!(f, "Failed to export as JSON string"),
            OpenApiError::FileWrite => write!(f, "Failed to write to file"),
        }
    }
}

/// Create OpenAPI 3.0 Specification File
#[cfg(not(tarpaulin_include))] // Exclude from coverage, doesn't appear to be a way to make this fail to generate a JSON
pub fn generate_openapi_spec<T>(target: &str) -> Result<(), OpenApiError>
where
    T: OpenApi,
{
    let output = T::openapi().to_pretty_json().map_err(|e| {
        rest_error!("failed to export as JSON string: {e}");
        OpenApiError::Json
    })?;

    std::fs::write(target, output).map_err(|e| {
        rest_error!("failed to write to file: {e}");
        OpenApiError::FileWrite
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_openapi_spec() {
        let target = "/nonsense/";
        let error = generate_openapi_spec::<ApiDoc>(target).unwrap_err();
        assert_eq!(error, OpenApiError::FileWrite);

        // TODO(R5): Is it possible to make the JSON export fail?
        // #[derive(OpenApi)]
        // #[openapi(
        //     paths(invalid)
        // )]
        // struct InvalidApi;
        // let error = generate_openapi_spec::<InvalidApi>("test.json").unwrap_err();
        // assert_eq!(error, OpenApiError::Json);
    }
}
