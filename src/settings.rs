use crate::errors::SonioxLiveErrors;
use crate::types::device::SettingDeviceId;
use crate::types::languages::LanguageHint;
use crate::types::tracing::TracingLevel;
use eframe::egui::{Align2, Color32, Vec2, vec2};
use eframe::epaint::Rgba;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::Level;

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsApp {
    pub(crate) language_hints: Vec<LanguageHint>,
    pub(crate) context: String,
    pub(crate) api_key: String,
    pub(crate) target_language: LanguageHint,
    pub(crate) enable_translate: bool,
    pub(crate) enable_high_priority: bool,
    pub(crate) enable_speakers: bool,
    pub(crate) level: TracingLevel,
    pub(crate) offset: (f32, f32),
    pub(crate) anchor: usize,
    pub(crate) font_size: usize,
    pub(crate) background_color: [f32; 4],
    pub(crate) text_color: [f32; 3],
    pub(crate) max_blocks: usize,
    pub(crate) device_id: Option<SettingDeviceId>,
    pub(crate) log_to_file: bool,
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self {
            language_hints: vec![LanguageHint::default()],
            context: String::from("some kind context"),
            api_key: String::new(),
            target_language: LanguageHint::default(),
            enable_translate: false,
            enable_high_priority: true,
            enable_speakers: true,
            level: TracingLevel::Info,
            offset: (0.0, -30.0),
            anchor: 7,
            font_size: 18,
            background_color: [0., 0., 0., 0.], // transparent
            text_color: [255., 255., 0.],       // yellow
            max_blocks: 3,
            device_id: None,
            log_to_file: false,
        }
    }
}

impl SettingsApp {
    pub fn new(path: &str) -> Result<Self, SonioxLiveErrors> {
        let path = Path::new(path);
        if !path.exists() {
            let s = Self::default();
            let content = toml::to_string(&s)?;
            std::fs::write(path, content)?;
            return Ok(s);
        }

        let content = std::fs::read_to_string(path)?;
        let s = toml::from_str(&content)?;
        Ok(s)
    }

    pub fn language_hints(&self) -> Arc<[LanguageHint]> {
        Arc::from(&*self.language_hints)
    }

    pub fn context(&self) -> Arc<str> {
        Arc::from(&*self.context)
    }

    pub fn api_key(&self) -> Arc<str> {
        Arc::from(&*self.api_key)
    }

    pub fn target_language(&self) -> LanguageHint {
        self.target_language
    }

    pub fn enable_speakers(&self) -> bool {
        self.enable_speakers
    }

    pub fn enable_translate(&self) -> bool {
        self.enable_translate
    }

    pub fn enable_high_priority(&self) -> bool {
        self.enable_high_priority
    }

    pub fn font_size(&self) -> f32 {
        self.font_size as f32
    }

    pub fn max_blocks(&self) -> usize {
        self.max_blocks
    }

    pub fn device_id(&self) -> Option<&SettingDeviceId> {
        self.device_id.as_ref()
    }
    
    pub fn log_to_file(&self) -> bool {
        self.log_to_file
    }

    pub fn level(&self) -> Level {
        Level::from(self.level)
    }

    pub fn text_color(&self) -> Color32 {
        let color = self.text_color;
        Rgba::from_rgb(color[0], color[1], color[2]).into()
    }

    pub fn background_color(&self) -> Color32 {
        let color = self.background_color;
        Rgba::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]).into()
    }

    pub fn get_anchor(&self) -> (Align2, Vec2) {
        let align = match self.anchor {
            0 => Align2::LEFT_TOP,
            1 => Align2::CENTER_TOP,
            2 => Align2::RIGHT_TOP,
            3 => Align2::LEFT_CENTER,
            4 => Align2::CENTER_CENTER,
            5 => Align2::RIGHT_CENTER,
            6 => Align2::LEFT_BOTTOM,
            7 => Align2::CENTER_BOTTOM,
            8 => Align2::RIGHT_BOTTOM,
            _ => Align2::CENTER_BOTTOM,
        };
        (align, vec2(self.offset.0, self.offset.1))
    }

    pub fn save(&self, path: &str) -> Result<(), SonioxLiveErrors> {
        let toml_string = toml::to_string(self)?;
        std::fs::write(path, toml_string)?;

        Ok(())
    }
}
