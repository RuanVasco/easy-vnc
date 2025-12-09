use gtk4::prelude::*;
use gtk4::Application;

mod model;
mod service;
mod ui;
mod config;

const APP_ID: &str = "com.github.RuanVasco.easy-remote";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(ui::build);
    app.run();
}
