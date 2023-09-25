//! Main function starting the server and initializing dependencies.

use dotenv::dotenv;
use log::info;
use svc_contact::config::Config;
use svc_contact::grpc;

/// Main entry point: starts gRPC Server on specified address and port
#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Will use default config settings if no environment vars are found.
    let config = Config::from_env().unwrap_or_default();

    dotenv().ok();
    {
        let log_cfg: &str = config.log_config.as_str();
        if let Err(e) = log4rs::init_file(log_cfg, Default::default()) {
            panic!("(logger) could not parse {}. {}", log_cfg, e);
        }
    }

    let _ = tokio::spawn(grpc::server::grpc_server(config)).await;

    info!("(main) server shutdown.");
    Ok(())
}
