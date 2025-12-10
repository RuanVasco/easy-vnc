use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct VncConnection {
    #[serde(rename = "@label")]
    pub label: String,
    #[serde(rename = "@ip")]
    pub ip: String,
    #[serde(rename = "@port")]
    pub port: u16,
}

impl VncConnection {
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
