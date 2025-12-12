mod service;

use gdk_pixbuf;
use gdk4;
use glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Picture};
use service::webrtc_viewer::WebRtcViewer;
use std::io::{self, BufRead, Cursor};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = async_channel::unbounded();
    let viewer = WebRtcViewer::new(tx).await?;

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut line = String::new();
        handle.read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        buffer.push_str(&line);
    }

    let answer = viewer.handle_offer(buffer).await?;

    println!("\n-------------------------------------------------");
    println!("2. Copie este SDP ANSWER e cole no Cliente:");
    println!("-------------------------------------------------");
    println!("{}", answer);
    println!("-------------------------------------------------\n");

    println!("Aguardando vídeo...");

    while let Ok(frame_data) = rx.recv().await {
        println!("Recebi frame JPEG! Tamanho: {} bytes", frame_data.len());
    }

    Ok(())
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Easy Remote - Técnico (Viewer)")
        .default_width(800)
        .default_height(600)
        .build();

    let picture = Picture::new();
    picture.set_content_fit(gtk4::ContentFit::Contain);
    window.set_child(Some(&picture));

    let (tx, rx) = async_channel::unbounded();

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            println!("Iniciando WebRTC...");

            let viewer = match WebRtcViewer::new(tx).await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Erro ao criar viewer: {}", e);
                    return;
                }
            };

            println!("\n--- MODO MANUAL ---");
            println!("Cole o OFFER do Windows aqui e dê ENTER duas vezes:");

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });
    });

    let picture_clone = picture.clone();

    glib::MainContext::default().spawn_local(async move {
        println!("Aguardando frames de vídeo...");

        while let Ok(frame_data) = rx.recv().await {
            let stream = Cursor::new(frame_data);

            match gdk_pixbuf::Pixbuf::from_read(stream) {
                Ok(pixbuf) => {
                    let texture = gdk::Texture::for_pixbuf(&pixbuf);
                    picture_clone.set_paintable(Some(&texture));
                }
                Err(e) => {
                    eprintln!("Erro ao decodificar frame JPEG: {}", e);
                }
            }
        }
    });

    window.present();
}
