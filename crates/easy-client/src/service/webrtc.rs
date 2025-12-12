use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use std::sync::Arc;
use tokio::sync::Mutex;

use webrtc::{
    api::{
        APIBuilder, interceptor_registry::register_default_interceptors, media_engine::MediaEngine,
    },
    data_channel::{RTCDataChannel, data_channel_state::RTCDataChannelState},
    peer_connection::{
        RTCPeerConnection,
        configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        sdp::{sdp_type::RTCSdpType, session_description::RTCSessionDescription},
    },
};

use local_ip_address::local_ip;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice_transport::ice_candidate_type::RTCIceCandidateType;
use webrtc::ice_transport::ice_gatherer_state::RTCIceGathererState;

pub struct WebRtcClient {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
}

impl WebRtcClient {
    pub async fn new() -> Result<Self> {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;

        let mut setting_engine = SettingEngine::default();

        let my_ip = local_ip()?;

        setting_engine.set_nat_1to1_ips(vec![my_ip.to_string()], RTCIceCandidateType::Host);

        let registry = register_default_interceptors(
            webrtc::interceptor::registry::Registry::new(),
            &mut media_engine,
        )?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .with_setting_engine(setting_engine)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![],
            ..Default::default()
        };

        let peer_connection = api.new_peer_connection(config).await?;

        let data_channel_ref = Arc::new(Mutex::new(None));
        let data_channel_clone = data_channel_ref.clone();

        let data_channel = peer_connection.create_data_channel("video", None).await?;

        let dc_clone = data_channel.clone();
        data_channel.on_open(Box::new(move || {
            println!("Data Channel 'video' aberto!");

            Box::pin(async move {
                let mut guard = data_channel_clone.lock().await;
                *guard = Some(dc_clone);
            })
        }));

        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                println!("WebRTC Connection State has changed: {}", s);
                Box::pin(async {})
            },
        ));

        Ok(Self {
            peer_connection: Arc::new(peer_connection),
            data_channel: data_channel_ref,
        })
    }

    pub async fn create_offer(&self) -> Result<String> {
        let offer = self.peer_connection.create_offer(None).await?;

        self.peer_connection
            .set_local_description(offer.clone())
            .await?;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
        let tx = Arc::new(Mutex::new(Some(tx)));

        self.peer_connection
            .on_ice_gathering_state_change(Box::new(move |state| {
                println!("Estado da coleta de IPs (ICE): {}", state);
                if state == RTCIceGathererState::Complete {
                    let tx_clone = tx.clone();
                    Box::pin(async move {
                        let mut tx_guard = tx_clone.lock().await;
                        if let Some(sender) = tx_guard.take() {
                            let _ = sender.send(()).await;
                        }
                    })
                } else {
                    Box::pin(async {})
                }
            }));

        let _ = tokio::time::timeout(tokio::time::Duration::from_secs(10), rx.recv()).await;
        let local_desc = self.peer_connection.local_description().await.unwrap();

        Ok(local_desc.sdp)
    }

    pub async fn start_streaming(
        &self,
        receiver: async_channel::Receiver<crate::service::capture::VideoFrame>,
    ) {
        loop {
            let maybe_dc = {
                let guard = self.data_channel.lock().await;
                guard.clone()
            };

            if let Some(dc) = maybe_dc {
                if dc.ready_state() == RTCDataChannelState::Open {
                    if let Ok(frame) = receiver.recv().await {
                        let expected_size = (frame.width * frame.height * 4) as usize;
                        let received_size = frame.data.len();

                        if received_size < 16 {
                            eprintln!("ERRO: Buffer muito pequeno para ser inspecionado!");
                            continue;
                        }

                        if received_size != expected_size {
                            eprintln!(
                                "AVISO: TAMANHO DE BUFFER INCORRETO. Esperado: {}, Recebido: {}",
                                expected_size, received_size
                            );
                            continue;
                        }

                        let jpg_bytes = tokio::task::spawn_blocking(
                            move || -> Result<Vec<u8>, image::ImageError> {
                                let mut buffer = Vec::new();
                                let mut cursor = std::io::Cursor::new(&mut buffer);
                                let mut encoder = JpegEncoder::new_with_quality(&mut cursor, 60);

                                let raw_img = image::RgbaImage::from_raw(
                                    frame.width,
                                    frame.height,
                                    frame.data,
                                )
                                .expect("Falha na criação da imagem crua. Buffer inválido.");

                                let rgb_img = image::DynamicImage::ImageRgba8(raw_img).to_rgb8();

                                encoder.encode(
                                    &rgb_img,
                                    rgb_img.width(),
                                    rgb_img.height(),
                                    image::ExtendedColorType::Rgb8,
                                )?;

                                Ok(buffer)
                            },
                        )
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("Erro na compressão JPEG: {}", e);
                            Ok(Vec::new())
                        })
                        .unwrap_or_default();

                        if !jpg_bytes.is_empty() {
                            let max_chunk_size = 16384;

                            for (i, chunk) in jpg_bytes.chunks(max_chunk_size).enumerate() {
                                let is_last_chunk =
                                    i == jpg_bytes.chunks(max_chunk_size).count() - 1;

                                let flag = if is_last_chunk { 1u8 } else { 0u8 };
                                let mut payload = vec![flag];
                                payload.extend_from_slice(chunk);

                                let _ = dc.send(&bytes::Bytes::from(payload)).await;
                            }
                        }
                    }
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }

    pub async fn handle_answer(&self, sdp_answer: String) -> anyhow::Result<()> {
        let mut desc = RTCSessionDescription::default();
        desc.sdp_type = RTCSdpType::Answer;
        desc.sdp = sdp_answer;

        self.peer_connection.set_remote_description(desc).await?;
        Ok(())
    }
}
