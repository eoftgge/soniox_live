pub mod connection;
pub mod session;

use async_trait::async_trait;
use std::collections::VecDeque;
use tungstenite::{Bytes, Message};

use connection::SonioxConnection;
use session::{SonioxSessionReader, SonioxSessionWriter};
use crate::types::soniox::{SonioxTranscriptionMessage, SonioxTranscriptionRequest};
use crate::stt::prelude::{SttProvider, SttEvent, SttError, TranscriptData};

const ERROR_CODES_RECONNECT: &[usize] = &[408, 502, 503];
const URL: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
const MODEL: &str = "stt-rt-v4";

pub struct SonioxAdapter {
    request: SonioxTranscriptionRequest,
    writer: Option<SonioxSessionWriter>,
    reader: Option<SonioxSessionReader>,

    current_sentence: String,
    current_speaker: Option<String>,
    event_queue: VecDeque<SttEvent>,
}

impl SonioxAdapter {
    pub fn new(request: SonioxTranscriptionRequest) -> Self {
        Self {
            request,
            writer: None,
            reader: None,
            current_sentence: String::new(),
            current_speaker: None,
            event_queue: VecDeque::new(),
        }
    }
}

#[async_trait]
impl SttProvider for SonioxAdapter {
    async fn connect(&mut self) -> Result<(), SttError> {
        let conn = SonioxConnection::connect(URL).await
            .map_err(|_| SttError::ConnectionLost)?;
        let (w, r) = conn.into_session(&self.request)
            .await
            .map_err(|_| SttError::ConnectionLost)?;
        self.writer = Some(w);
        self.reader = Some(r);
        self.current_sentence.clear();
        self.event_queue.clear();
        Ok(())
    }

    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError> {
        let writer = self.writer.as_mut().ok_or(SttError::ConnectionLost)?;
        writer
            .send_bytes(Bytes::copy_from_slice(audio)).await
            .map_err(|_| SttError::ConnectionLost)
    }

    async fn recv_event(&mut self) -> Result<SttEvent, SttError> {
        if let Some(event) = self.event_queue.pop_front() {
            return Ok(event);
        }

        let reader = self.reader.as_mut().ok_or(SttError::ConnectionLost)?;
        let writer = self.writer.as_mut().ok_or(SttError::ConnectionLost)?;
        loop {
            let msg = reader.recv_message().await.map_err(|e| {
                tracing::error!("WS Error/EOF: {}", e);
                SttError::ConnectionLost
            })?;

            match msg {
                Message::Text(txt) => {
                    let parsed_msg: SonioxTranscriptionMessage = serde_json::from_str(&txt)
                        .map_err(|e| SttError::FatalAPIError(format!("JSON parse error: {}", e)))?;

                    match parsed_msg {
                        SonioxTranscriptionMessage::Response(r) => {
                            let mut has_interim = false;

                            for token in r.tokens {
                                if token.translation_status.as_deref() == Some("original") {
                                    continue;
                                }

                                self.current_speaker = token.speaker.clone();
                                self.current_sentence.push_str(&token.text);

                                if token.is_final {
                                    self.event_queue.push_back(SttEvent::Transcript(TranscriptData {
                                        text: self.current_sentence.clone(),
                                        is_final: true,
                                        speaker: self.current_speaker.clone(),
                                    }));
                                    self.current_sentence.clear();
                                } else {
                                    has_interim = true;
                                }
                            }

                            if has_interim && !self.current_sentence.is_empty() {
                                self.event_queue.push_back(SttEvent::Transcript(TranscriptData {
                                    text: self.current_sentence.clone(),
                                    is_final: false,
                                    speaker: self.current_speaker.clone(),
                                }));
                            }

                            if let Some(event) = self.event_queue.pop_front() {
                                return Ok(event);
                            }
                        }
                        SonioxTranscriptionMessage::Error(e) => {
                            return if ERROR_CODES_RECONNECT.contains(&e.error_code) {
                                Err(SttError::RecoverableAPIError(e.error_message))
                            } else {
                                Err(SttError::FatalAPIError(e.error_message))
                            }
                        }
                    }
                }
                Message::Ping(data) => {
                    let _ = writer.send_pong(data).await;
                }
                Message::Close(_) => {
                    tracing::warn!("Server sent Close frame");
                    return Ok(SttEvent::Disconnected);
                }
                _ => continue,
            }
        }
    }
}