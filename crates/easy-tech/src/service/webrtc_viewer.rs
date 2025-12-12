use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_gatherer_state::RTCIceGathererState;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use local_ip_address::local_ip;
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice_transport::ice_candidate_type::RTCIceCandidateType;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

pub struct WebRtcViewer {
    peer_connection: Arc<RTCPeerConnection>,
}

impl WebRtcViewer {
    pub async fn new(frame_sender: async_channel::Sender<Vec<u8>>) -> Result<Self> {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;

        let mut setting_engine = SettingEngine::default();

        setting_engine.set_interface_filter(Box::new(|iface| {
            println!("Interface detectada: {} (IPs: {:?})", iface, iface);
            true
        }));

        let my_ip = local_ip()?;

        setting_engine.set_nat_1to1_ips(vec![my_ip.to_string()], RTCIceCandidateType::Host);

        let registry = register_default_interceptors(Registry::new(), &mut media_engine)?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .with_setting_engine(setting_engine)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![],
            ..Default::default()
        };

        let frame_buffer_ref = Arc::new(Mutex::new(Vec::<u8>::new()));

        let peer_connection = api.new_peer_connection(config).await?;

        peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let sender_clone = frame_sender.clone();
            let frame_buffer_clone = frame_buffer_ref.clone();

            d.on_message(Box::new(move |msg| {
                let sender_for_async = sender_clone.clone();
                let buffer_for_async = frame_buffer_clone.clone();

                Box::pin(async move {
                    let data = msg.data.to_vec();
                    let is_last_chunk = data[0] == 1u8;
                    let chunk_data = &data[1..];

                    let mut buffer_guard = buffer_for_async.lock().await;
                    buffer_guard.extend_from_slice(chunk_data);

                    if is_last_chunk {
                        println!(
                            "🚀 FRAME REMONTADO! Tamanho Total: {} bytes",
                            buffer_guard.len()
                        );
                        let full_frame = buffer_guard.drain(..).collect::<Vec<u8>>();

                        let _ = sender_for_async.send_blocking(full_frame);
                    } else {
                        println!(
                            "Recebendo fragmento... buffer: {} bytes",
                            buffer_guard.len()
                        );
                    }
                })
            }));
            Box::pin(async {})
        }));

        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                println!("Viewer State: {}", s);
                Box::pin(async {})
            },
        ));

        Ok(Self {
            peer_connection: Arc::new(peer_connection),
        })
    }

    pub async fn handle_offer(&self, sdp_offer: String) -> Result<String> {
        let mut desc = RTCSessionDescription::default();
        desc.sdp_type = RTCSdpType::Offer;
        desc.sdp = sdp_offer;
        self.peer_connection.set_remote_description(desc).await?;

        let answer = self.peer_connection.create_answer(None).await?;
        self.peer_connection.set_local_description(answer).await?;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
        let tx = Arc::new(Mutex::new(Some(tx)));
        self.peer_connection
            .on_ice_gathering_state_change(Box::new(move |state| {
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
        let _ = tokio::time::timeout(tokio::time::Duration::from_secs(2), rx.recv()).await;

        let local_desc = self.peer_connection.local_description().await.unwrap();
        Ok(local_desc.sdp)
    }
}
