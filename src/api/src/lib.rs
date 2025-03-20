use std::sync::Arc;

use anyhow::Result;
use service::new_router;
use sidewinder_core::exec::driver::Driver;
use tokio::net::TcpListener;

pub(crate) mod error;
pub mod service;

pub async fn run_api(port: i32, driver: Arc<dyn Driver>) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tracing::info!(
        addr = "0.0.0.0",
        port,
        "listener opened"
    );

    Ok(axum::serve(listener, new_router(driver)).await?)
}