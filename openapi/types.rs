/// Types used for REST communication with the svc-cargo server

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Signup Request Type
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
#[derive(ToSchema, IntoParams)]
pub struct SignupRequest {
    /// The email to use
    pub email: String,

    /// The display name to use
    pub display_name: String,
}
