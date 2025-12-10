use gtk4::Application;
use gtk4::prelude::*;

mod config;
mod model;
mod service;
mod ui;

const APP_ID: &str = "com.github.RuanVasco.easy-client";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(ui::build);
    app.run();
}
