#[derive(Debug, Clone)]
pub enum Error {
    WaylandDetected,
    ConnectionRefused,
    Timeout,
    // PermissionDenied,
    // Unknown(String),
}

impl Error {
    pub fn from_log(log: &str) -> Option<Self> {
        let log_lower = log.to_lowercase();

        if log_lower.contains("wayland") {
            return Some(Self::WaylandDetected);
        }
        if log_lower.contains("failed to connect") || log_lower.contains("connection refused") {
            return Some(Self::ConnectionRefused);
        }
        if log_lower.contains("timed out") || log_lower.contains("timeout") {
            return Some(Self::Timeout);
        }

        None
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::WaylandDetected => {
                "Error: Incompatible graphics system (Wayland detected).".to_string()
            }
            Self::ConnectionRefused => {
                "Error: Connection refused. Is the technician listening?".to_string()
            }
            Self::Timeout => "Error: Connection timed out.".to_string(),
            // Self::PermissionDenied => "Error: Permission denied.".to_string(),
            // Self::Unknown(msg) => format!("Unexpected Error: {}", msg),
        }
    }
}
