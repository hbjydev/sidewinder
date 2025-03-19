use anyhow::Result;
use sidewinder_core::{exec::driver::Driver, server::ServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let docker = sidewinder_docker::Docker::new()?;

    docker
        .run_server(
            String::from("abc123"),
            ServerConfig {
                bind_address: String::from("0.0.0.0"),
                bind_port: Some(2001),
                public_address: None,
                public_port: 2001,
                ..Default::default()
            },
        )
        .await?;

    Ok(())
}
