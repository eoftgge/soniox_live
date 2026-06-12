#[derive(Debug, Clone)]
pub struct TranscriptData {
    pub text: String,
    pub is_final: bool,
    pub speaker: Option<String>,
}