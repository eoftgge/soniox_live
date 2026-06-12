use crate::settings::SettingsApp;
use crate::stt::provider::{SttProvider};
use crate::stt::event::SttError;
use crate::stt::adapters::soniox::SonioxAdapter;

use cpal::StreamConfig;
use crate::stt::adapters::soniox::request::create_request;

pub fn create_stt_provider(
    settings: &SettingsApp,
    audio_config: &StreamConfig,
) -> Result<Box<dyn SttProvider>, SttError> {
    let request = create_request(settings, audio_config)
        .map_err(|e| SttError::FatalAPIError(format!("Failed to build Soniox request: {}", e)))?;
    let adapter = SonioxAdapter::new(request);
    Ok(Box::new(adapter))
}