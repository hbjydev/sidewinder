pub mod driver;

/// An execution that was spawned by an execution driver.
pub struct Execution {
    /// The execution ID, set by the chosen driver.
    pub id: String,

    /// The server ID this execution was spawned for.
    pub server_id: String,
}