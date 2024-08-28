//! Re-export of used objects

pub use super::client as contact;
pub use super::service::Client as ContactServiceClient;
pub use contact::ContactClient;

pub use lib_common::grpc::Client;
