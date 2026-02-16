use crate::errors::SonioxLiveErrors;
use crate::settings::SettingsApp;
use crate::transcription::service::TranscriptionService;
use crate::transcription::store::TranscriptionStore;
use crate::types::device::MappableAvailableDevices;
use eframe::egui::{Context, ViewportCommand, WindowLevel};

pub struct StateManager {
    app_state: AppState,
    pending_state: Option<PendingState>,
}

#[derive(Clone, Copy)]
pub enum PendingState {
    Settings,
    Overlay,
}

pub enum AppState {
    Settings,
    Overlay(TranscriptionService),
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            app_state: AppState::Settings,
            pending_state: Some(PendingState::Settings),
        }
    }

    pub fn switch(&mut self, new_state: PendingState) {
        self.pending_state = Some(new_state);
    }

    pub fn resolve(
        &mut self,
        ctx: &Context,
        store: &mut TranscriptionStore,
        settings: &SettingsApp,
        devices: &MappableAvailableDevices,
    ) -> Result<(), SonioxLiveErrors> {
        let Some(resolved) = self.pending_state.take() else {
            return Ok(());
        };

        resolved.apply_window_state(ctx, settings.enable_high_priority());
        match resolved {
            PendingState::Settings => self.app_state = AppState::Settings,
            PendingState::Overlay => {
                let ctx = ctx.clone();
                let service = TranscriptionService::start(ctx, settings, devices)?;
                store.resize(settings.max_blocks);
                self.app_state = AppState::Overlay(service);
            }
        }
        Ok(())
    }

    pub fn app_state(&self) -> &AppState {
        &self.app_state
    }

    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.app_state
    }
}

impl PendingState {
    pub fn apply_window_state(&self, ctx: &Context, enable_high_priority: bool) {
        match self {
            Self::Settings => {
                ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
                ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(false));
                ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
            }
            Self::Overlay => {
                ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(ViewportCommand::Transparent(true));
                ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));
                ctx.send_viewport_cmd(ViewportCommand::Maximized(true));

                if enable_high_priority {
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                }
            }
        }
    }
}
