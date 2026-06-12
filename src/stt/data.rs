#[derive(Debug, Clone)]
pub struct TranscriptData {
    pub text: String,
    pub is_final: bool,
    pub speaker_id: Option<String>,
    pub translation: Option<String>,
    pub language_code: Option<String>,
}