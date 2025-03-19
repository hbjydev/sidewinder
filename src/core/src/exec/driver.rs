use std::future::Future;

use anyhow::Result;

use crate::server::ServerConfig;

use super::Execution;

pub trait Driver {
    fn run_server(&self, id: String, config: ServerConfig) -> impl Future<Output = Result<super::Execution>>;
    fn list_servers(&self) -> impl Future<Output = Result<Vec<super::Execution>>>;
    fn stop_server(&self, execution: Execution) -> impl Future<Output = Result<()>>;
    fn get_logs(&self, execution: Execution) -> impl Future<Output = Result<()>>;
}