use gtk4::glib::BoxedAnyObject;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box, Button, Frame, Label, ListItem, ListView,
    NoSelection, Orientation, PolicyType, ScrolledWindow, SignalListItemFactory, gio, glib,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

use crate::config::Entries;
use crate::model::VncConnection;
use crate::service::vnc_launcher::VncLauncher;

pub fn build(app: &Application) {
    let store = gio::ListStore::new::<BoxedAnyObject>();
    let entries = Entries::load();

    for connection in entries {
        store.append(&BoxedAnyObject::new(connection));
    }

    let selection_model = NoSelection::new(Some(store));

    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let status_label = Label::builder()
        .label("Pronto para conectar.")
        .margin_top(10)
        .margin_bottom(10)
        .use_markup(true)
        .build();

    let status_frame = Frame::builder()
        .child(&status_label)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .build();

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

            let (sender, receiver) = async_channel::bounded::<()>(1);

            let btn = btn_ui.clone();
            let lbl = lbl_ui.clone();

            glib::MainContext::default().spawn_local(async move {
                while receiver.recv().await.is_ok() {
                    btn.set_sensitive(true);
                    btn.set_label("Conectar");
                    lbl.set_label("Conexão encerrada.");
                }
            });

            // Feedback visual imediato
            status_click.set_label(&format!("Conectando em <b>{}</b>...", conn_ref.label));

            // 3. EXECUTAR O PROCESSO
            match VncLauncher::launch(&mut conn_ref) {
                Ok(mut child) => {
                    button_click.set_sensitive(false);
                    button_click.set_label("Rodando...");
                    status_click.set_label("Conexão ativa. Aguardando técnico...");

                    // 4. THREAD DE BACKGROUND (Só para esperar o VNC)
                    // Movemos apenas o sender (que é thread-safe) e o child
                    thread::spawn(move || {
                        let _ = child.wait(); // Bloqueia esta thread secundária

                        // Envia sinal para o 'spawn_local' lá em cima (send_blocking pois estamos em thread comum)
                        let _ = sender.send_blocking(());
                    });
                }
                Err(e) => {
                    status_click.set_label(&format!("<span foreground='red'>Erro: {}</span>", e));
                }
            }
        });
    });

    let list_view = ListView::new(Some(selection_model), Some(factory));

    container.append(&status_frame);
    container.append(&list_view);

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .min_content_height(400)
        .child(&container)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Easy Remote")
        .default_width(350)
        .default_height(500)
        .resizable(false)
        .child(&scrolled_window)
        .build();

    window.present();
}
