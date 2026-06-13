pub mod connection;
pub mod request;
pub mod session;
pub mod types;

use async_trait::async_trait;
use std::collections::VecDeque;
use tungstenite::{Bytes, Message};

use crate::stt::adapters::soniox::types::SonioxTranscriptionToken;
use crate::stt::prelude::{SttError, SttEvent, SttProvider, TranscriptData};
use connection::SonioxConnection;
use session::{SonioxSessionReader, SonioxSessionWriter};
use types::{SonioxTranscriptionMessage, SonioxTranscriptionRequest};

const ERROR_CODES_RECONNECT: &[usize] = &[408, 502, 503];
const URL: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
const MODEL: &str = "stt-rt-v4";

pub struct SonioxAdapter {
    request: SonioxTranscriptionRequest,
    writer: Option<SonioxSessionWriter>,
    reader: Option<SonioxSessionReader>,

    event_queue: VecDeque<SttEvent>,
}

impl SonioxAdapter {
    pub fn new(request: SonioxTranscriptionRequest) -> Self {
        Self {
            request,
            writer: None,
            reader: None,
            event_queue: VecDeque::new(),
        }
    }

    async fn handle_ws_message(&mut self, msg: Message) -> Result<Option<SttEvent>, SttError> {
        match msg {
            Message::Text(txt) => self.handle_text_message(&txt),
            Message::Ping(data) => {
                if let Some(writer) = self.writer.as_mut() {
                    let _ = writer.send_pong(data).await;
                }
                Ok(None)
            }
            Message::Close(_) => {
                tracing::warn!("Server sent Close frame");
                Ok(Some(SttEvent::Disconnected))
            }
            _ => Ok(None),
        }
    }
    fn handle_text_message(&mut self, txt: &str) -> Result<Option<SttEvent>, SttError> {
        let parsed_msg: SonioxTranscriptionMessage = serde_json::from_str(txt)
            .map_err(|e| SttError::FatalAPIError(format!("JSON parse error: {}", e)))?;

        match parsed_msg {
            SonioxTranscriptionMessage::Response(r) => {
                self.enqueue_tokens(r.tokens);
                Ok(self.event_queue.pop_front())
            }
            SonioxTranscriptionMessage::Error(e) => {
                if ERROR_CODES_RECONNECT.contains(&e.error_code) {
                    Err(SttError::RecoverableAPIError(e.error_message))
                } else {
                    Err(SttError::FatalAPIError(e.error_message))
                }
            }
        }
    }

    fn enqueue_tokens(&mut self, tokens: Vec<SonioxTranscriptionToken>) {
        let mut final_text = String::new();
        let mut interim_text = String::new();
        let mut current_speaker = None;

        for token in tokens {
            if token.translation_status.as_deref() == Some("original") {
                continue;
            }

            let token_speaker = token.speaker.clone();

            if current_speaker.is_some() && current_speaker != token_speaker {
                self.flush_buffers(&mut final_text, &mut interim_text, &current_speaker);
            }

            current_speaker = token_speaker;
            if token.is_final {
                final_text.push_str(&token.text);
            } else {
                interim_text.push_str(&token.text);
            }
        }

        self.flush_buffers(&mut final_text, &mut interim_text, &current_speaker);
    }

    fn flush_buffers(
        &mut self,
        final_text: &mut String,
        interim_text: &mut String,
        speaker: &Option<String>,
    ) {
        if !final_text.is_empty() {
            self.event_queue
                .push_back(SttEvent::Transcript(TranscriptData {
                    text: std::mem::take(final_text),
                    is_final: true,
                    speaker: speaker.clone(),
                }));
        }
        if !interim_text.is_empty() {
            self.event_queue
                .push_back(SttEvent::Transcript(TranscriptData {
                    text: std::mem::take(interim_text),
                    is_final: false,
                    speaker: speaker.clone(),
                }));
        }
    }
}

#[async_trait]
impl SttProvider for SonioxAdapter {
    async fn connect(&mut self) -> Result<(), SttError> {
        let conn = SonioxConnection::connect(URL)
            .await
            .map_err(|_| SttError::ConnectionLost)?;
        let (w, r) = conn
            .into_session(&self.request)
            .await
            .map_err(|_| SttError::ConnectionLost)?;

        self.writer = Some(w);
        self.reader = Some(r);
        self.event_queue.clear();
        Ok(())
    }

    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError> {
        let writer = self.writer.as_mut().ok_or(SttError::ConnectionLost)?;
        writer
            .send_bytes(Bytes::copy_from_slice(audio))
            .await
            .map_err(|_| SttError::ConnectionLost)
    }

    async fn recv_event(&mut self) -> Result<SttEvent, SttError> {
        if let Some(event) = self.event_queue.pop_front() {
            return Ok(event);
        }

        loop {
            let msg = {
                let reader = self.reader.as_mut().ok_or(SttError::ConnectionLost)?;
                reader.recv_message().await.map_err(|e| {
                    tracing::error!("WS Error/EOF: {}", e);
                    SttError::ConnectionLost
                })?
            };

            if let Some(event) = self.handle_ws_message(msg).await? {
                return Ok(event);
            }
        }
    }
}
