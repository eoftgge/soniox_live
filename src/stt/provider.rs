pub trait SttProvider: Send + Sync {
    async fn connect(&mut self);
    async fn send(&mut self, audio: &[u8]);
    async fn recv_event(&mut self);
}