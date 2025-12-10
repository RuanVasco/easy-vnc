use gtk4::glib::BoxedAnyObject;
use gtk4::pango::EllipsizeMode;
use gtk4::{
    Align, Application, ApplicationWindow, Box, Button, Label, ListItem, ListView, NoSelection,
    Orientation, PolicyType, ScrolledWindow, SignalListItemFactory, gio, glib,
};
use gtk4::{CssProvider, prelude::*};
use gtk4::{gdk, style_context_add_provider_for_display};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::Entries;
use crate::model::{Error, VncConnection, VncEvent};
use crate::service::vnc_launcher::VncLauncher;

use std::io::{BufRead, BufReader};

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("../resources/style.css"));

    if let Some(display) = gdk::Display::default() {
        style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn update_status(label: &Label, text: &str, css_class: &str) {
    label.set_label(text);
    for class in &[
        "status-ready",
        "status-loading",
        "status-success",
        "status-error",
    ] {
        label.remove_css_class(class);
    }
    label.add_css_class(css_class);
}

pub fn build(app: &Application) {
    load_css();

    let store = gio::ListStore::new::<BoxedAnyObject>();
    let entries = Entries::load();
    for connection in entries {
        store.append(&BoxedAnyObject::new(connection));
    }
    let selection_model = NoSelection::new(Some(store));

    let container = Box::builder().orientation(Orientation::Vertical).build();

    let status_label = Label::builder()
        .label("Ready to connect.")
        .xalign(0.0)
        .css_classes(vec!["status-bar", "status-ready"])
        .wrap(true)
        .wrap_mode(gtk4::pango::WrapMode::WordChar)
        .lines(2)
        .ellipsize(EllipsizeMode::End)
        .max_width_chars(40)
        .hexpand(true)
        .build();

    let status_box = Box::builder().orientation(Orientation::Horizontal).build();
    status_box.append(&status_label);

    let status_label_factory = status_label.clone();

    let factory = SignalListItemFactory::new();

    factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let hbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(10)
            .margin_end(10)
            .build();

        let label = Label::builder().xalign(0.0).hexpand(true).build();
        let button = Button::builder()
            .label("Conectar")
            .valign(Align::Center)
            .build();

        hbox.append(&label);
        hbox.append(&button);
        item.set_child(Some(&hbox));
    });

    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let hbox = item.child().and_downcast::<Box>().unwrap();
        let label = hbox.first_child().unwrap().downcast::<Label>().unwrap();
        let button = hbox.last_child().unwrap().downcast::<Button>().unwrap();

        let entry = item.item().and_downcast::<BoxedAnyObject>().unwrap();
        let vnc_conn = entry.borrow::<VncConnection>();

        label.set_label(&format!("{} ({})", vnc_conn.label, vnc_conn.ip));

        let conn_wrapper = Rc::new(RefCell::new(vnc_conn.clone()));

        let button_click = button.clone();
        let status_click = status_label_factory.clone();

        let btn_ui = button.clone();
        let lbl_ui = status_label_factory.clone();

        button.connect_clicked(move |_| {
            let mut conn_ref = conn_wrapper.borrow_mut();

            let (sender, receiver) = async_channel::unbounded::<VncEvent>();

            let btn = btn_ui.clone();
            let lbl = lbl_ui.clone();

            glib::MainContext::default().spawn_local(async move {
                while let Ok(msg) = receiver.recv().await {
                    match msg {
                        VncEvent::ConnectionError(error) => {
                            update_status(&lbl, &error, "status-error");
                        }
                        VncEvent::Log(_text) => {}
                        VncEvent::Finished => {
                            btn.set_sensitive(true);
                            btn.set_label("Conectar");
                            update_status(&lbl, "Disconnected. Ready.", "status-ready");
                        }
                    }
                }
            });

            update_status(
                &status_click,
                &format!("Connecting to {}...", conn_ref.label),
                "status-loading",
            );

            match VncLauncher::launch(&mut conn_ref) {
                Ok(mut child) => {
                    button_click.set_sensitive(false);
                    button_click.set_label("Conectando...");
                    update_status(
                        &status_click,
                        "Connection active. Waiting for technician...",
                        "status-success",
                    );

                    if let Some(stderr) = child.stderr.take() {
                        let sender_log = sender.clone();
                        thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for l in reader.lines().map_while(Result::ok) {
                                if let Some(error_enum) = Error::from_log(&l) {
                                    let _ = sender_log.send_blocking(VncEvent::ConnectionError(
                                        error_enum.user_message(),
                                    ));
                                } else {
                                    let _ = sender_log.send_blocking(VncEvent::Log(l));
                                }
                            }
                        });
                    }

                    thread::spawn(move || {
                        let begin = Instant::now();
                        let limit = Duration::from_secs(60);

                        loop {
                            match child.try_wait() {
                                Ok(Some(_status)) => {
                                    let _ = sender.send_blocking(VncEvent::Finished);
                                    break;
                                }
                                Ok(None) | Err(_) => {
                                    if begin.elapsed() > limit {
                                        let _ = child.kill();
                                        let _ = child.wait();

                                        let _ = sender.send_blocking(VncEvent::ConnectionError(
                                            "Tempo esgotado (60s). Tente novamente.".into(),
                                        ));
                                        let _ = sender.send_blocking(VncEvent::Finished);
                                        break;
                                    }

                                    thread::sleep(Duration::from_millis(500));
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    update_status(
                        &status_click,
                        &format!("Erro ao iniciar: {}", e),
                        "status-error",
                    );
                }
            }
        });
    });

    let list_view = ListView::new(Some(selection_model), Some(factory));

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .min_content_height(400)
        .child(&list_view)
        .vexpand(true)
        .build();

    container.append(&scrolled_window);
    container.append(&status_box);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Easy Remote")
        .default_width(350)
        .default_height(500)
        .resizable(false)
        .build();

    window.set_child(Some(&container));

    window.present();
}
