mod service;

use service::webrtc_viewer::WebRtcViewer;
use std::io::{self, BufRead};

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
