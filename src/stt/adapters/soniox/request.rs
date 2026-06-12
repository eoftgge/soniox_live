use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;
use crate::stt::adapters::soniox::MODEL;
use super::types::{SonioxTranscriptionRequest, SonioxTranslationObject};
use cpal::StreamConfig;

pub(crate) fn create_request(
    settings: &SettingsApp,
    stream_config: &StreamConfig,
) -> Result<SonioxTranscriptionRequest, OmniSttErrors> {
    let mut request = SonioxTranscriptionRequest {
        api_key: settings.api_key(),
        model: MODEL,
        audio_format: "pcm_s16le",
        sample_rate: Some(16000),
        num_channels: Some(1),
        context: Some(settings.context()),
        language_hints: settings.language_hints(),
        enable_speaker_diarization: Some(settings.enable_speakers()),
        ..Default::default()
    };
    if settings.enable_translate {
        request.translation = Some(SonioxTranslationObject {
            r#type: "one_way",
            target_language: Some(settings.target_language()),
            ..Default::default()
        });
    }

    Ok(request)
}
