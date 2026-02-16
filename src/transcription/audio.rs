use crate::errors::SonioxLiveErrors;
use crate::transcription::utils::convert_audio_chunk;
use crate::types::audio::AudioSample;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct AudioSession {
    stream: Stream,
    config: StreamConfig,
}

impl AudioSession {
    pub fn new(config: StreamConfig, stream: Stream) -> Self {
        Self { config, stream }
    }

    pub fn open(
        device: Device,
        tx_audio: Sender<AudioSample>,
        mut rx_recycle: Receiver<AudioSample>,
    ) -> Result<Self, SonioxLiveErrors> {
        let config = device.default_output_config()?.config();
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = match rx_recycle.try_recv() {
                    Ok(sample) => sample,
                    Err(_) => Vec::with_capacity(data.len()),
                };
                convert_audio_chunk(data, &mut buffer);
                match tx_audio.try_send(buffer) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {
                        tracing::debug!("Audio buffer is full");
                    }
                    Err(TrySendError::Closed(_)) => {
                        tracing::debug!("Capture channel closed");
                    }
                }
            },
            |err| {
                tracing::error!("Error in audio callback: {}", err);
            },
            None,
        )?;

        Ok(Self::new(config, stream))
    }

    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    pub fn play(&self) -> Result<(), cpal::PlayStreamError> {
        self.stream.play()
    }

    pub fn pause(&self) -> Result<(), cpal::PauseStreamError> {
        self.stream.pause()
    }
}
