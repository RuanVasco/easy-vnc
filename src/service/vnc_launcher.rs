use crate::model::VncConnection;
use std::env;
use std::process::Command;

pub struct VncLauncher;

impl VncLauncher {
    pub fn launch(connection: &VncConnection) {
        let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let target = connection.address();
        let status;
        
        if session_type.to_lowercase().contains("wayland") {
            status = Command::new("wayvnc")
                .arg("-c")
                .arg(&target)
                .spawn();
        } else {
            status = Command::new("x11vnc")
                .arg("-connect")
                .arg(&target)
                .arg("-display").arg(":0")
                .arg("-ncache").arg("10")
                .arg("-once")
                .arg("-desktop")
                .arg("-nopw")
                .spawn();
        }
           
        match status {
            Ok(_) => {},
            Err(e) => eprintln!("Falha ao executar o comando: {}", e),
        }
    }
}