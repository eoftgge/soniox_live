use crate::errors::SonioxLiveErrors;
use crate::soniox::URL;
use crate::soniox::action::StreamAction;
use crate::soniox::connection::SonioxConnection;
use crate::soniox::session::{SonioxSessionReader, SonioxSessionWriter};
use crate::types::audio::AudioSample;
use crate::types::events::SonioxEvent;
use crate::types::soniox::{SonioxTranscriptionMessage, SonioxTranscriptionRequest};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;
use tungstenite::{Bytes, Message};
use crate::transcription::utils::is_silent;

const MAX_RETRIES: u32 = 5;
const RECONNECT_DELAY: u64 = 1000;
const ERROR_CODES_RECONNECT: &[usize] = &[408, 502, 503];
const HANGOVER_CHUNKS_LIMIT: usize = 5;

pub(crate) struct SonioxWorker {
    rx_audio: Receiver<AudioSample>,
    tx_recycle: Sender<AudioSample>,
    tx_event: Sender<SonioxEvent>,
}

impl SonioxWorker {
    pub(crate) fn new(
        rx_audio: Receiver<AudioSample>,
        tx_recycle: Sender<AudioSample>,
        tx_event: Sender<SonioxEvent>,
    ) -> Self {
        Self {
            rx_audio,
            tx_event,
            tx_recycle,
        }
    }

    pub(crate) async fn run(
        mut self,
        request: &SonioxTranscriptionRequest,
    ) -> Result<(), SonioxLiveErrors> {
        let mut retry_count = 0;
        let mut flag_first_connection = true;

        loop {
            let first_packet = if retry_count == 0 {
                let Some(packet) = self.wait_first_packet().await else {
                    return Ok(());
                };
                packet
            } else {
                Vec::new()
            };

            tracing::debug!("Connecting to Soniox... (Attempt {})", retry_count + 1);
            let conn_result = SonioxConnection::connect(URL).await;
            let conn = match conn_result {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Connection failed: {}", e);
                    if self.handle_reconnect(&mut retry_count).await.is_err() {
                        return Err(SonioxLiveErrors::ConnectionLost);
                    }
                    continue;
                }
            };
            let session_result = conn.into_session(request).await;
            let (mut writer, reader) = match session_result {
                Ok((w, r)) => (w, r),
                Err(e) => {
                    tracing::warn!("Handshake failed: {}", e);
                    if self.handle_reconnect(&mut retry_count).await.is_err() {
                        return Err(SonioxLiveErrors::ConnectionLost);
                    }
                    continue;
                }
            };

            tracing::info!("Connected to Soniox");
            retry_count = 0;
            if !first_packet.is_empty()
                && let Err(e) = self.handle_audio(first_packet, &mut writer).await
            {
                tracing::error!("Failed to send initial audio: {}", e);
                continue;
            }

            let _ = self
                .tx_event
                .send(SonioxEvent::Connected(flag_first_connection))
                .await;
            if flag_first_connection {
                flag_first_connection = false;
            }

            let action = self.run_session_loop(writer, reader).await;
            match action {
                StreamAction::Stop => {
                    tracing::info!("Worker stopped normally");
                    return Ok(());
                }
                StreamAction::Reconnect => {
                    tracing::warn!("Session ended. Going to standby...");
                }
                StreamAction::Continue => {}
            }
        }
    }

    async fn run_session_loop(
        &mut self,
        mut writer: SonioxSessionWriter,
        mut reader: SonioxSessionReader,
    ) -> StreamAction {
        let mut hangover_counter = 0;

        loop {
            tokio::select! {
                audio_opt = self.rx_audio.recv() => {
                    let Some(mut buffer) = audio_opt else {
                        tracing::debug!("Audio channel closed by app. Stopping worker.");
                        return StreamAction::Stop;
                    };

                    if !is_silent(&buffer) {
                        hangover_counter = HANGOVER_CHUNKS_LIMIT;
                    } else if hangover_counter > 0 {
                        hangover_counter = hangover_counter.saturating_sub(1);
                    } else {
                        buffer.clear();
                        let _ = self.tx_recycle.send(buffer).await;
                        continue;
                    }

                    if let Err(e) = self.handle_audio(buffer, &mut writer).await {
                        tracing::error!("Failed to send audio: {}", e);
                        return StreamAction::Reconnect;
                    }
                }
                msg_result = reader.recv_message() => {
                    let action = match msg_result {
                        Ok(m) => self.handle_ws_message(m, &mut writer).await,
                        Err(e) => {
                            tracing::error!("WS Error/EOF: {}", e);
                            let _ = self.tx_event.send(SonioxEvent::from("Connection interrupted")).await;
                            return StreamAction::Reconnect;
                        }
                    };
                    if !matches!(action, StreamAction::Continue) {
                        return action;
                    }
                }
            }
        }
    }

    async fn handle_audio(
        &self,
        mut buffer: AudioSample,
        writer: &mut SonioxSessionWriter,
    ) -> Result<(), SonioxLiveErrors> {
        if buffer.is_empty() {
            return Ok(());
        }

        let slice: &[u8] = bytemuck::cast_slice(&buffer);
        writer.send_bytes(Bytes::copy_from_slice(slice)).await?;
        buffer.clear();
        let _ = self.tx_recycle.send(buffer).await;
        Ok(())
    }

    async fn handle_ws_message(
        &self,
        message: Message,
        writer: &mut SonioxSessionWriter,
    ) -> StreamAction {
        match message {
            Message::Text(txt) => match serde_json::from_str::<SonioxTranscriptionMessage>(&txt) {
                Ok(parsed_msg) => self.process_transcription_msg(parsed_msg).await,
                Err(e) => {
                    tracing::warn!("JSON parse error: {}. Raw: {}", e, txt);
                    StreamAction::Continue
                }
            },
            Message::Ping(data) => {
                let _ = writer.send_pong(data).await;
                StreamAction::Continue
            }
            Message::Close(_) => {
                tracing::warn!("Server sent Close frame");
                StreamAction::Reconnect
            }
            _ => StreamAction::Continue,
        }
    }

    async fn process_transcription_msg(&self, msg: SonioxTranscriptionMessage) -> StreamAction {
        match msg {
            SonioxTranscriptionMessage::Response(r) => {
                if self
                    .tx_event
                    .send(SonioxEvent::Transcription(r))
                    .await
                    .is_err()
                {
                    return StreamAction::Stop;
                }
                StreamAction::Continue
            }
            SonioxTranscriptionMessage::Error(e)
                if ERROR_CODES_RECONNECT.contains(&e.error_code) =>
            {
                tracing::warn!("Temporary API Error {}: {}", e.error_code, e.error_message);
                StreamAction::Reconnect
            }
            SonioxTranscriptionMessage::Error(e) => {
                tracing::error!("Fatal API Error {}: {}", e.error_code, e.error_message);
                let _ = self
                    .tx_event
                    .send(SonioxEvent::Error(SonioxLiveErrors::API(
                        e.error_code,
                        e.error_message,
                    )))
                    .await;
                StreamAction::Stop
            }
        }
    }

    async fn handle_reconnect(&self, retry_count: &mut u32) -> Result<(), ()> {
        sleep(Duration::from_millis(RECONNECT_DELAY)).await;
        *retry_count += 1;

        if *retry_count > MAX_RETRIES {
            let _ = self
                .tx_event
                .send(SonioxEvent::Error(SonioxLiveErrors::ConnectionLost))
                .await;
            return Err(());
        }
        Ok(())
    }

    async fn wait_first_packet(&mut self) -> Option<AudioSample> {
        tracing::debug!("Waiting for speech to connect to Soniox...");

        loop {
            match self.rx_audio.recv().await {
                Some(packet) if !is_silent(&packet) => {
                    return Some(packet);
                },
                Some(mut packet) => {
                    packet.clear();
                    let _ = self.tx_recycle.send(packet).await;
                },
                None => {
                    tracing::info!("Audio channel closed by app. Stopping worker.");
                    return None;
                }
            }
        }
    }
}
