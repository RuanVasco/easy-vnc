use crate::model::VncConnection;
use std::process::{Child, Command, Stdio};
use std::{env, io};

pub struct VncLauncher;

impl VncLauncher {
    pub fn launch(connection: &mut VncConnection) -> io::Result<Child> {
        let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let target = connection.address();

        let mut cmd = if session_type.to_lowercase().contains("wayland") {
            let mut c = Command::new("wayvnc");
            c.arg("-c").arg(&target);
            c
        } else {
            let mut c = Command::new("x11vnc");
            c.arg("-display")
                .arg(":0")
                .arg("-connect")
                .arg(&target)
                .arg("-ncache")
                .arg("10")
                .arg("-once")
                .arg("-nopw");
            c
        };

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        cmd.spawn()
    }
}
