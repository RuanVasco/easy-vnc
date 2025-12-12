mod service;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Picture};
use service::webrtc_viewer::WebRtcViewer;
use std::io::{self, BufRead, Cursor};
use tokio::runtime::Runtime;

const APP_ID: &str = "com.github.RuanVasco.easy-remote.tech";

#[tokio::main]
async fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Easy Remote - Técnico")
        .default_width(800)
        .default_height(600)
        .build();

    let picture = Picture::new();

    picture.set_can_shrink(true);

    window.set_child(Some(&picture));

    let (tx, rx) = async_channel::unbounded();

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let viewer = match WebRtcViewer::new(tx).await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Erro fatal ao criar viewer: {}", e);
                    return;
                }
            };

            println!("\n=== MODO TÉCNICO ===");
            println!("1. Cole o OFFER do Windows aqui e dê ENTER duas vezes:");

            let mut buffer = String::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            loop {
                let mut line = String::new();
                handle.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                buffer.push_str(&line);
            }

            println!("Gerando resposta...");
            match viewer.handle_offer(buffer).await {
                Ok(answer) => {
                    println!("\n-------------------------------------------------");
                    println!("2. COPIE ESTA ANSWER E COLE NO WINDOWS:");
                    println!("-------------------------------------------------");
                    println!("{}", answer);
                    println!("-------------------------------------------------\n");
                }
                Err(e) => eprintln!("Erro ao processar offer: {}", e),
            }

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });
    });

    let picture_clone = picture.clone();

    glib::MainContext::default().spawn_local(async move {
        println!("Interface Gráfica: Loop de vídeo iniciado!");
        let mut frame_count = 0;

        loop {
            let match_result = rx.recv().await;

            match match_result {
                Ok(mut frame_data) => {
                    let mut dropped = 0;
                    while !rx.is_empty() {
                        if let Ok(newer_frame) = rx.try_recv() {
                            frame_data = newer_frame;
                            dropped += 1;
                        } else {
                            break;
                        }
                    }

                    if dropped > 0 {
                        println!("Skipped {} old frames to catch up", dropped);
                    }

                    let stream = Cursor::new(frame_data);
                    match gdk_pixbuf::Pixbuf::from_read(stream) {
                        Ok(pixbuf) => {
                            let texture = gdk4::Texture::for_pixbuf(&pixbuf);
                            picture_clone.set_paintable(Some(&texture));

                            frame_count += 1;
                            if frame_count % 30 == 0 {
                                println!("Renderizando frame #{}", frame_count);
                            }
                        }
                        Err(e) => eprintln!("Erro JPEG: {}", e),
                    }
                }
                Err(_) => {
                    println!("Canal fechado. Encerrando vídeo.");
                    break;
                }
            }
        }
    });

    window.present();
}
