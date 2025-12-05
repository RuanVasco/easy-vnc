use serde::Deserialize;
use std::process::Command;

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
    pub fn connect(&self) {
        let target = format!("{}::{}", self.ip, self.port);

        let status = Command::new("xtightvncviewer")
            .arg(&target)
            .spawn();

        match status {
            Ok(_) => println!("Cliente VNC iniciado com sucesso!"),
            Err(e) => eprintln!("Falha ao executar o comando: {}", e),
        }
    }
}
