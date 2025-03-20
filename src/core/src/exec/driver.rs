use std::fmt::Debug;

use anyhow::Result;

use crate::server::ServerConfig;
use crate::async_trait;

use super::Execution;

#[async_trait]
pub trait Driver: Debug + Send + Sync + 'static {
    async fn run_server(
        &self,
        id: String,
        config: ServerConfig,
    ) -> Result<super::Execution>;

    async fn list_servers(&self) -> Result<Vec<super::Execution>>;
    async fn stop_server(&self, execution: Execution) -> Result<()>;
    async fn get_logs(&self, execution: Execution) -> Result<()>;
}
