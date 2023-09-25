//! gRPC client helpers implementation

// TODO(R4) import other microservices' GRPC clients
// use svc_storage_client_grpc::prelude::Clients;
pub use tonic::transport::Channel;

/// Struct to hold all gRPC client connections
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct GrpcClients {
    // TODO(R4) clients here
    // pub storage: Clients
}

impl Default for GrpcClients {
    /// Creates default clients
    fn default() -> Self {
        // let storage = Clients::new(config.storage_host_grpc, config.storage_port_grpc);

        GrpcClients {
            // TODO(R4) - add other clients here
            // storage
        }
    }
}
