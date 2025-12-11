#[derive(Default, NwgUi)]
pub struct WindowsApp {
    #[nwg_control(size: (350, 500), position: (300, 300), title: "Easy Remote Client", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [WindowsApp::sair] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 5)]
    layout: nwg::GridLayout,

    #[nwg_control(collection: vec![])]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, row_span: 5)]
    entry_list: nwg::ListBox<String>,

    #[nwg_control(text: "Conectar")]
    #[nwg_layout_item(layout: layout, col: 0, row: 5)]
    #[nwg_events( OnButtonClick: [WindowsApp::conectar] )]
    connect_btn: nwg::Button,

    dados: RefCell<Vec<crate::model::VncConnection>>,
}
