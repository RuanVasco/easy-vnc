use async_channel::Sender;
use std::thread;
use std::time::{Duration, Instant};
use xcap::Monitor;

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct ScreenCapture;

impl ScreenCapture {
    pub fn start(sender: Sender<VideoFrame>) {
        thread::spawn(move || {
            let monitors = Monitor::all().unwrap_or_else(|e| {
                eprintln!("Erro ao listar monitores: {}", e);
                vec![]
            });

            if let Some(monitor) = monitors.first() {
                let frame_duration = Duration::from_millis(33);

                loop {
                    let start = Instant::now();

                    match monitor.capture_image() {
                        Ok(image) => {
                            let width = image.width();
                            let height = image.height();

                            let raw_pixels = image.into_raw();

                            let frame = VideoFrame {
                                width,
                                height,
                                data: raw_pixels,
                            };

                            if sender.send_blocking(frame).is_err() {
                                println!("canal fechado");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Erro: {}", e);
                        }
                    }

                    let elapsed = start.elapsed();
                    if elapsed < frame_duration {
                        thread::sleep(frame_duration - elapsed);
                    }
                }
            } else {
                eprintln!("Nenhum monitor encontrado!");
            }
        });
    }
}
