//! REST API handlers for the contact service.

/// Public types needed to communicate with the REST interface
pub mod rest_types {
    include!("../../../../openapi/types.rs");
}

pub mod health;
pub mod user;
