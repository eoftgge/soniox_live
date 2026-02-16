use crate::gui::overlay::draw_subtitles;
use crate::gui::settings::show_settings_window;
use crate::gui::state::{AppState, StateManager};
use crate::settings::SettingsApp;
use crate::transcription::service::TranscriptionService;
use crate::transcription::store::TranscriptionStore;
use crate::types::events::SonioxEvent;
use eframe::egui::{
    Align, Area, Context, Id, Layout, Order, ViewportCommand, Visuals, WindowLevel,
};
use eframe::{App, Frame};
use egui_notify::Toasts;
use std::time::Duration;
use tracing_appender::non_blocking::WorkerGuard;
use crate::types::device::MappableAvailableDevices;

fn process_events(
    service: &mut TranscriptionService,
    store: &mut TranscriptionStore,
    toasts: &mut Toasts,
) {
    while let Ok(event) = service.receiver.try_recv() {
        match event {
            SonioxEvent::Transcription(r) => {
                store.update(r);
            }
            SonioxEvent::Warning(s) => {
                toasts
                    .warning(s.to_string())
                    .duration(Duration::from_secs(4))
                    .closable(false);
            }
            SonioxEvent::Error(e) => {
                toasts
                    .error(e.to_string())
                    .duration(Duration::from_secs(4))
                    .closable(false);
            }
            SonioxEvent::Connected(flag_first_connection) => {
                store.ensure_separator();
                if flag_first_connection {
                    toasts
                        .info("Connected to Soniox!")
                        .duration(Duration::from_secs(4))
                        .closable(false);
                }
            }
        };
    }
}

pub struct SubtitlesApp {
    settings: SettingsApp,
    store: TranscriptionStore,
    toasts: Toasts,
    manager: StateManager,
    frame_counter: u64,
    devices: MappableAvailableDevices,
    _guard: WorkerGuard,
}

impl SubtitlesApp {
    pub fn new(settings: SettingsApp, guard: WorkerGuard) -> Self {
        Self {
            store: TranscriptionStore::new(settings.max_blocks()),
            toasts: Toasts::new(),
            manager: StateManager::new(),
            settings,
            frame_counter: 0,
            devices: MappableAvailableDevices::from_default_host(),
            _guard: guard,
        }
    }
}

impl App for SubtitlesApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if let Err(err) = self.manager.resolve(ctx, &mut self.store, &self.settings, &self.devices) {
            self.toasts.error(format!("{:?}", err)).closable(false);
        }
        let manager = &mut self.manager;

        match manager.app_state_mut() {
            AppState::Config => show_settings_window(ctx, &mut self.settings, manager, &mut self.toasts, &mut self.devices),
            AppState::Overlay(service) => {
                let timeout = Duration::from_secs(15);
                let ctx_for_plan = ctx.clone();
                self.store.clear_if_silent(timeout);
                self.store.schedule(ctx_for_plan, timeout);

                process_events(service, &mut self.store, &mut self.toasts);
                if self.settings.enable_high_priority() && self.frame_counter >= 100 {
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                    self.frame_counter = 0;
                }
                let (anchor, offset) = self.settings.get_anchor();
                Area::new(Id::from("subtitles_area"))
                    .anchor(anchor, offset)
                    .order(Order::Foreground)
                    .show(ctx, |ui| {
                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            draw_subtitles(
                                ui,
                                &self.store,
                                self.settings.font_size(),
                                self.settings.text_color(),
                                self.settings.get_background_color(),
                            );
                        });
                    });

                self.frame_counter += 1;
            }
        }

        self.toasts.show(ctx);
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
