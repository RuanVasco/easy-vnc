use crate::ui::UserInterface;

mod config;
mod service;
mod ui;

fn main() {
    let app = UserInterface::new();
    app.run();
}
