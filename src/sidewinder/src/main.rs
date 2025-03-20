use std::sync::Arc;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliDriver {
    Docker,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The port to run the API on.
    #[arg(short, long, default_value_t = 1234)]
    port: i32,

    /// The driver to use to control server instances.
    #[arg(short, long, value_enum, default_value = "docker")]
    driver: CliDriver,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("info"))
                .unwrap()
        )
        .init();

    let args = Cli::parse();

    let driver= match args.driver {
        CliDriver::Docker => sidewinder_docker::Docker::new()?,
    };

    Ok(sidewinder_api::run_api(args.port, Arc::new(driver)).await?)
}