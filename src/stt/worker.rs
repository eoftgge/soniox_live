use crate::stt::event::SttEvent;
use crate::stt::provider::SttProvider;
use crate::types::audio::AudioSample;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct GenericSttWorker {
    rx_audio: Receiver<AudioSample>,
    tx_recycle: Sender<AudioSample>,
    tx_event: Sender<SttEvent>,
    hangover_chunks_limit: usize,
    vad_threshold: u32,
    provider: Box<dyn SttProvider>,
}