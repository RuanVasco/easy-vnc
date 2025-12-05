use gtk4::glib::BoxedAnyObject;
use gtk4::{Align, Label, ListItem, ListView, Orientation, PolicyType, SignalListItemFactory, NoSelection, gio, prelude::*};
use gtk4::{Application, ApplicationWindow, Button, ScrolledWindow, Box};

use crate::config::Entries;
use crate::model::VncConnection;

pub fn build(app: &Application) {
    let store = gio::ListStore::new::<BoxedAnyObject>();
    let entries = Entries::load();

    for connection in entries {
        let obj = BoxedAnyObject::new(connection);
        store.append(&obj);
    }

    let selection_model = NoSelection::new(Some(store));

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

        let label = Label::builder()
            .xalign(0.0)
            .hexpand(true)
            .build();

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

        let conn_clone = vnc_conn.clone();

        button.connect_clicked(move |_| {
            conn_clone.connect();
        });
    });

    let list_view = ListView::new(Some(selection_model), Some(factory));

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .min_content_height(400)
        .child(&list_view)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Easy VNC")
        .default_width(350)
        .default_height(500)
        .resizable(false)
        .child(&scrolled_window)
        .build();

    window.present();
}
