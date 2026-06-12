use crate::stt::event::{SttError, SttEvent};
use async_trait::async_trait;

#[async_trait]
pub trait SttProvider: Send + Sync {
    async fn connect(&mut self) -> Result<(), SttError>;
    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError>;
    async fn recv_event(&mut self) -> Result<SttEvent, SttError>;
}
