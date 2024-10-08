//! gRPC server implementation

/// module generated from proto/svc-contact-grpc.proto
mod grpc_server {
    #![allow(unused_qualifications, missing_docs)]
    tonic::include_proto!("grpc");
}
pub use grpc_server::rpc_service_server::{RpcService, RpcServiceServer};
pub use grpc_server::{CargoConfirmationRequest, CargoConfirmationResponse};
pub use grpc_server::{ReadyRequest, ReadyResponse};

use crate::shutdown_signal;
use crate::Config;

use std::fmt::Debug;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

/// struct to implement the gRPC server functions
#[derive(Debug, Default, Copy, Clone)]
pub struct ServerImpl {}

#[cfg(not(feature = "stub_server"))]
#[tonic::async_trait]
impl RpcService for ServerImpl {
    /// Returns ready:true when service is available
    async fn is_ready(
        &self,
        request: Request<ReadyRequest>,
    ) -> Result<Response<ReadyResponse>, Status> {
        grpc_info!("contact server.");
        grpc_debug!("[{:?}].", request);
        let response = ReadyResponse { ready: true };
        Ok(Response::new(response))
    }

    /// Returns a response with the cargo confirmation
    async fn cargo_confirmation(
        &self,
        request: Request<CargoConfirmationRequest>,
    ) -> Result<Response<CargoConfirmationResponse>, Status> {
        grpc_info!("contact server.");
        grpc_debug!("[{:?}].", request);
        let response = super::api::cargo::cargo_confirmation(request.into_inner()).await?;
        Ok(Response::new(response))
    }
}

#[cfg(feature = "stub_server")]
#[tonic::async_trait]
impl RpcService for ServerImpl {
    async fn is_ready(
        &self,
        request: Request<ReadyRequest>,
    ) -> Result<Response<ReadyResponse>, Status> {
        grpc_warn!("(MOCK) contact server.");
        grpc_debug!("(MOCK) [{:?}].", request);
        let response = ReadyResponse { ready: true };
        Ok(Response::new(response))
    }

    async fn cargo_confirmation(
        &self,
        request: Request<CargoConfirmationRequest>,
    ) -> Result<Response<CargoConfirmationResponse>, Status> {
        grpc_warn!("(MOCK) contact server.");
        grpc_debug!("(MOCK) [{:?}].", request);
        let response = CargoConfirmationResponse { success: true };
        Ok(Response::new(response))
    }
}

/// Starts the grpc servers for this microservice using the provided configuration
///
/// # Examples
/// ```
/// use svc_contact::grpc::server::grpc_server;
/// use svc_contact::Config;
/// async fn example() -> Result<(), tokio::task::JoinError> {
///     let config = Config::default();
///     tokio::spawn(grpc_server(config, None)).await
/// }
/// ```
pub async fn grpc_server(config: Config, shutdown_rx: Option<tokio::sync::oneshot::Receiver<()>>) {
    grpc_debug!("entry.");

    // Grpc Server
    let grpc_port = config.docker_port_grpc;
    let full_grpc_addr: SocketAddr = match format!("[::]:{}", grpc_port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            grpc_error!("Failed to parse gRPC address: {}", e);
            return;
        }
    };

    let imp = ServerImpl::default();
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<RpcServiceServer<ServerImpl>>()
        .await;

    //start server
    grpc_info!("Starting gRPC services on: {}", full_grpc_addr);
    match Server::builder()
        .add_service(health_service)
        .add_service(RpcServiceServer::new(imp))
        .serve_with_shutdown(full_grpc_addr, shutdown_signal("grpc", shutdown_rx))
        .await
    {
        Ok(_) => grpc_info!("gRPC server running at: {}", full_grpc_addr),
        Err(e) => {
            grpc_error!("Could not start gRPC server: {}", e);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_server_is_ready() {
        lib_common::logger::get_log_handle().await;
        ut_info!("Start.");

        let imp = ServerImpl::default();
        let result = imp.is_ready(Request::new(ReadyRequest {})).await;
        assert!(result.is_ok());
        let result: ReadyResponse = result.unwrap().into_inner();
        assert_eq!(result.ready, true);

        ut_info!("Success.");
    }

    #[tokio::test]
    async fn test_grpc_server_start_and_shutdown() {
        use tokio::time::{sleep, Duration};
        lib_common::logger::get_log_handle().await;
        ut_info!("start");

        let config = Config::default();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Start the grpc server
        tokio::spawn(grpc_server(config, Some(shutdown_rx)));

        // Give the server time to get through the startup sequence (and thus code)
        sleep(Duration::from_secs(1)).await;

        // Shut down server
        assert!(shutdown_tx.send(()).is_ok());

        ut_info!("success");
    }

    #[tokio::test]
    #[cfg(feature = "stub_server")]
    async fn test_grpc_cargo_confirmation() {
        lib_common::logger::get_log_handle().await;
        ut_info!("start");

        let imp = ServerImpl::default();
        let result = imp
            .cargo_confirmation(Request::new(CargoConfirmationRequest {
                itinerary_id: String::from(lib_common::uuid::Uuid::new_v4()),
                parcel_id: String::from(lib_common::uuid::Uuid::new_v4()),
            }))
            .await;
        assert!(result.is_ok());
        let result: CargoConfirmationResponse = result.unwrap().into_inner();
        assert!(result.success);

        ut_info!("success");
    }
}
