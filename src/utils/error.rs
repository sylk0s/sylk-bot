// Error connecting to the MC interface
pub struct ConnectionError {
    reason: String,
}

impl ConnectionError {
    // new error from a reason
    pub fn from(reason: &str) -> ConnectionError {
        ConnectionError { reason: String::from(reason) }
    }
}
