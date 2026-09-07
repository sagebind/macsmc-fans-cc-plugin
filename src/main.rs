mod service;
mod sys;
mod tuxedo_io;

use crate::{
    device_service::v1::device_service_server::DeviceServiceServer, service::TuxedoService,
};
use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, error, info};
use std::str::FromStr;
use systemd_journal_logger::{JournalLog, connected_to_journal};
use tokio::{
    net::UnixListener,
    signal::{self, unix::SignalKind},
};
use tokio_util::sync::CancellationToken;
use tonic::{codegen::tokio_stream::wrappers::UnixListenerStream, transport::Server};

pub const SERVICE_ID: &str = "tuxedo-infinitybook-gen10";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const ENV_CC_LOG: &str = "CC_LOG";

pub mod models {
    pub mod v1 {
        tonic::include_proto!("coolercontrol.models.v1");
    }
}
pub mod device_service {
    pub mod v1 {
        tonic::include_proto!("coolercontrol.device_service.v1");
    }
}

/// A CoolerControl Device Service Plugin
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[clap(short, long)]
    debug: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let run_token = setup_termination_signals();
    setup_logging()?;
    info!("Starting {SERVICE_ID} v{VERSION}");

    let service = TuxedoService::new();

    // The default socket path for device services requires privileged access. Using the following
    // will work for both privileged and non-privileged services.
    // Make sure it's also correct in the manifest.toml
    let uds_path = format!("/tmp/{SERVICE_ID}.sock");
    cleanup_uds(&uds_path).await;
    let uds = match UnixListener::bind(&uds_path) {
        Ok(listener) => listener,
        Err(err) => {
            error!(
                "Failed to bind to socket: {uds_path}. If using privileged access, \
                make sure the service is running as root."
            );
            return Err(err.into());
        }
    };
    let uds_stream = UnixListenerStream::new(uds);
    Server::builder()
        .add_service(DeviceServiceServer::new(service))
        .serve_with_incoming_shutdown(uds_stream, run_token.cancelled())
        .await?;
    cleanup_uds(&uds_path).await;

    Ok(())
}

/// The CoolerControl daemon will pass the current daemon's log level as an environment variable.
/// If it is not set, it will default to Info.
fn setup_logging() -> Result<()> {
    let args: Args = Args::parse();
    let log_level = if args.debug {
        LevelFilter::Debug
    } else if let Ok(log_lvl) = std::env::var(ENV_CC_LOG) {
        LevelFilter::from_str(&log_lvl).unwrap_or(LevelFilter::Info)
    } else {
        LevelFilter::Info
    };
    if connected_to_journal() {
        JournalLog::new()?
            .with_extra_fields(vec![("VERSION", VERSION)])
            .install()?;
        log::set_max_level(log_level);
    } else {
        env_logger::Builder::new().filter_level(log_level).init();
    }
    Ok(())
}

/// Sets up signal handlers for termination and interrupt signals,
/// and returns a `CancellationToken` that is triggered when any of
/// those signals are received, allowing the caller to handle the
/// signal gracefully.
///
/// # Errors
///
/// This function returns an error if there is a problem setting up
/// the signal handlers.
fn setup_termination_signals() -> CancellationToken {
    let run_token = CancellationToken::new();
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    let sigterm = async {
        signal::unix::signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    let sigint = async {
        signal::unix::signal(SignalKind::interrupt())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    let sigquit = async {
        signal::unix::signal(SignalKind::quit())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    let sig_run_token = run_token.clone();
    tokio::task::spawn(async move {
        tokio::select! {
            () = ctrl_c => {},
            () = sigterm => {},
            () = sigint => {},
            () = sigquit => {},
        }
        sig_run_token.cancel();
        info!("Shutting down");
    });
    run_token
}

/// Cleanup the UDS file if it exists
///
/// If a system goes down unexpectedly, an existing file can block a service restart
/// from binding to it again.
async fn cleanup_uds(uds_path: &str) {
    let _ = tokio::fs::remove_file(uds_path).await;
}
