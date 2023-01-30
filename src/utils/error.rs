pub struct ConnectionError {
    reason: String,
}

impl ConnectionError {
    pub fn from(reason: &str) -> ConnectionError {
        ConnectionError { reason: String::from(reason) }
    }
}
