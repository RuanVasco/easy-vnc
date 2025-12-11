use crate::{config::Entries, ui::windows::app_layout_ui::AppLayoutUi};
use easy_core::model::VncConnection;
use native_windows_derive::NwgUi;
use native_windows_gui::{self as nwg, NativeUi}; // Importe NativeUi!
use std::cell::RefCell;

#[derive(Default, NwgUi)]
pub struct AppLayout {
    #[nwg_control(size: (350, 500), position: (300, 300), title: "Easy Remote Client", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [AppLayout::on_window_close] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 5)]
    layout: nwg::GridLayout,

    #[nwg_control(collection: vec![])]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, row_span: 5)]
    entry_list: nwg::ListBox<String>,

    #[nwg_control(text: "Connect")]
    #[nwg_layout_item(layout: layout, col: 0, row: 5)]
    #[nwg_events( OnButtonClick: [AppLayout::on_connect_click] )]
    connect_btn: nwg::Button,

    #[nwg_control(text: "Ready.")]
    #[nwg_layout_item(layout: layout, col: 0, row: 6)]
    status_label: nwg::Label,

    data: RefCell<Vec<VncConnection>>,
}

impl AppLayout {
    fn on_window_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn on_connect_click(&self) {
        if let Some(index) = self.entry_list.selection() {
            let data = self.data.borrow();
            if let Some(conn) = data.get(index) {
                self.status_label
                    .set_text(&format!("Connecting to {}...", conn.label));
                self.connect_btn.set_enabled(false);
                nwg::simple_message("Connecting", &format!("Target: {}", conn.ip));
            }
        }
    }

    fn load_data(&self) {
        let entries = Entries::load();
        let titles: Vec<String> = entries
            .iter()
            .map(|e| format!("{} ({})", e.label, e.ip))
            .collect();

        self.entry_list.set_collection(titles);
        *self.data.borrow_mut() = entries;
    }
}

pub struct WindowsApp {
    _ui: AppLayoutUi,
}

impl WindowsApp {
    pub fn new() -> Self {
        nwg::init().expect("Failed to init NWG");
        nwg::Font::set_global_family("Segoe UI").ok();

        let ui = AppLayout::build_ui(Default::default()).expect("Failed to build UI");

        ui.load_data();

        Self { _ui: ui }
    }

    pub fn run(&self) {
        nwg::dispatch_thread_events();
    }
}
