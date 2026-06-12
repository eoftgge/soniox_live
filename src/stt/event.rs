use crate::stt::data::TranscriptData;

pub enum SttEvent {
    Connected(bool),
    Transcript(TranscriptData),
    Error(SttError),
    Disconnected,
}

pub enum SttError {
    ConnectionLost,
    FatalAPIError(String),
    RecoverableAPIError(String),
}