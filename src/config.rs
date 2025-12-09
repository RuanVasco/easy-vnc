use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::model::vnc_connection::VncConnection;

#[derive(Debug, Deserialize)]
pub struct Entries {
    #[serde(rename = "Entry", default)]
    pub entries: Vec<VncConnection>,
}

impl Entries {
    pub fn load() -> Vec<VncConnection> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "github", "easy-remote") {
            let config_dir = proj_dirs.config_dir();
            let config_file = config_dir.join("entries.xml");

            if config_file.exists() {
                return read_file(config_file);
            }
        }

        let system_file = PathBuf::from("/etc/easy-remote/entries.xml");
        if system_file.exists() {
            return read_file(system_file);
        }

        let dev_file = PathBuf::from("assets/entries.xml");
        if dev_file.exists() {
            return read_file(dev_file);
        }

        vec![]
    }
}

fn read_file(path: PathBuf) -> Vec<VncConnection> {
    let Ok(content) = fs::read_to_string(&path) else {
        return vec![];
    };

    let Ok(list) = quick_xml::de::from_str::<Entries>(&content) else {
        return vec![];
    };

    list.entries
}
