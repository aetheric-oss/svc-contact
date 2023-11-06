//! Re-export of used objects

pub use super::client as contact;
pub use super::service::Client as TemplateRustServiceClient;
pub use contact::TemplateRustClient;

pub use lib_common::grpc::Client;
