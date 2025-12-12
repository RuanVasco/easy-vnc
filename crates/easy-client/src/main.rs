use std::thread;

use crate::ui::UserInterface;

mod config;
mod service;
mod ui;

fn main() {
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        })
    });

    let app = UserInterface::new();
    app.run();
}
