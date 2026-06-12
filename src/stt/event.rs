pub enum SttEvent {
    Connected(bool),
    Transcript(),
    Error(SttError),
    Disconnected,
}

pub enum SttError {
    ConnectionLost,
    FatalAPIError(String),
    RecoverableAPIError,
}