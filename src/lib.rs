use crate::gui::app::SubtitlesApp;
use settings::SettingsApp;
use tracing::Level;

pub mod errors;
pub mod gui;
pub mod settings;
pub mod soniox;
pub mod transcription;
pub mod types;

pub const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

fn setup_tracing(level: Level, log_to_file: bool) -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("logs", "soniox.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(level)
        .with_ansi(false)
        .with_thread_names(true)
        .with_file(log_to_file)
        .with_line_number(true)
        .init();

    guard
}

pub fn initialize_app(settings: SettingsApp) -> SubtitlesApp {
    let level = settings.level();
    let guard = setup_tracing(level, settings.log_to_file);
    SubtitlesApp::new(settings, guard)
}
