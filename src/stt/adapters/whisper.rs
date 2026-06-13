use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tokio::time::sleep;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use crate::stt::data::TranscriptData;
use crate::stt::event::{SttError, SttEvent};
use crate::stt::provider::SttProvider;

pub struct WhisperAdapter {
    path: PathBuf,
    audio_tx: Option<Sender<Vec<f32>>>,
    event_rx: Option<Receiver<SttEvent>>,
}

impl WhisperAdapter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), audio_tx: None, event_rx: None }
    }
}

#[async_trait]
impl SttProvider for WhisperAdapter {
    async fn connect(&mut self) -> Result<(), SttError> {
        let ctx = WhisperContext::new_with_params(
            &self.path,
            WhisperContextParameters::default(),
        ).map_err(|e| SttError::FatalAPIError(format!("Failed to load model: {}", e)))?;
        let ctx = Arc::new(ctx);

        let (audio_tx, mut audio_rx) = channel::<Vec<f32>>(100);
        let (event_tx, event_rx) = channel::<SttEvent>(100);

        self.audio_tx = Some(audio_tx);
        self.event_rx = Some(event_rx);

        let event_tx_clone = event_tx.clone();

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let silence_timeout = Duration::from_millis(500);

            let _ = event_tx_clone.send(SttEvent::Connected(true)).await;
            loop {
                tokio::select! {
                    audio_chunk = audio_rx.recv() => {
                        match audio_chunk {
                            Some(mut chunk) => buffer.append(&mut chunk),
                            None => break,
                        }
                    }

                    _ = sleep(silence_timeout), if !buffer.is_empty() => {
                        let audio_to_process = buffer.clone();
                        buffer.clear();

                        let ctx_clone = ctx.clone();
                        let event_tx_inner = event_tx_clone.clone();

                        tokio::task::spawn_blocking(move || {
                            let mut state = ctx_clone.create_state().expect("Failed to create Whisper state");

                            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                            params.set_language(Some("ru"));
                            params.set_print_progress(false);
                            params.set_print_special(false);
                            params.set_print_realtime(false);

                            if state.full(params, &audio_to_process).is_ok() {
                                let mut text = String::new();
                                let num_segments = state.full_n_segments();

                                for i in 0..num_segments {
                                    if let Some(segment) = state.get_segment(i) {
                                        text.push_str(&segment.to_string());
                                    }
                                }

                                let text = text.trim().to_string();
                                if !text.is_empty() {
                                    let data = TranscriptData { text, is_final: true, speaker: None };
                                    let _ = event_tx_inner.blocking_send(SttEvent::Transcript(data));
                                }
                            }
                        });
                    }
                }
            }
        });

        Ok(())
    }

    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError> {
        if let Some(tx) = &self.audio_tx {
            let audio_i16: &[i16] = bytemuck::cast_slice(audio);
            let audio_f32: Vec<f32> = audio_i16
                .iter()
                .map(|&s| s as f32 / 32768.0)
                .collect();
            tx.send(audio_f32).await.map_err(|_| {
                SttError::FatalAPIError("Whisper audio channel closed".into())
            })?;
        }
        Ok(())
    }

    async fn recv_event(&mut self) -> Result<SttEvent, SttError> {
        if let Some(rx) = &mut self.event_rx {
            rx.recv().await.ok_or_else(|| {
                SttError::FatalAPIError("Whisper event channel closed".into())
            })
        } else {
            Err(SttError::FatalAPIError("Whisper provider not connected".into()))
        }
    }
}