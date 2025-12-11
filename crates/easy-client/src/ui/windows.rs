use crate::{config::Entries, ui::windows::app_layout_ui::AppLayoutUi};
use easy_core::model::VncConnection;
use native_windows_derive::NwgUi;
use native_windows_gui::{self as nwg, NativeUi};
use std::{cell::RefCell, thread};

use crate::service::capture::ScreenCapture;
use crate::service::webrtc::WebRtcClient;
use std::io::{self, BufRead};

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
                    .set_text(&"Handshake manual iniciada... Olhe o Terminal!".to_string());
                self.connect_btn.set_enabled(false);

                let _target_ip = conn.ip.clone();

                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();

                    rt.block_on(async {
                        println!("\n=== INICIANDO CONEXÃO WEBRTC (MODO MANUAL) ===");
                        let (tx, rx) = async_channel::unbounded();

                        ScreenCapture::start(tx);
                        println!("Captura de tela iniciada.");

                        let client = match WebRtcClient::new().await {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("Erro ao criar WebRTC: {}", e);
                                return;
                            }
                        };

                        let offer = client.create_offer().await.unwrap();

                        println!("\n-------------------------------------------------");
                        println!("1. COPIE ESTE CÓDIGO (OFFER) E COLE NO EASY-TECH:");
                        println!("-------------------------------------------------");
                        println!("{}", offer);
                        println!("-------------------------------------------------\n");

                        println!("2. Agora cole a RESPOSTA (ANSWER) do easy-tech aqui e dê ENTER duas vezes:");
                        
                        let mut buffer = String::new();
                        let stdin = io::stdin();
                        let mut handle = stdin.lock();
                        loop {
                            let mut line = String::new();
                            handle.read_line(&mut line).unwrap();
                            if line.trim().is_empty() { break; }
                            buffer.push_str(&line);
                        }

                        println!("Processando resposta...");
                        client.handle_answer(buffer).await.unwrap();

                        println!("Conexão estabelecida! Iniciando envio de vídeo...");
                        
                        client.start_streaming(rx).await;
                    })
                });
            }
        }

        let (tx, rx) = async_channel::unbounded();

        crate::service::capture::ScreenCapture::start(tx);        
    }

    fn load_data(&self) {
        let entries = Entries::load();
        let titles: Vec<String> = entries.iter().map(|e| e.to_string()).collect();

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
