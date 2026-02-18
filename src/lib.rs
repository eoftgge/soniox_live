use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

pub mod errors;
pub mod gui;
pub mod settings;
pub mod soniox;
pub mod transcription;
pub mod types;

pub fn setup_tracing(level: Level, log_to_file: bool) -> Option<WorkerGuard> {
    let (writer, guard) = if log_to_file {
        let file_appender = tracing_appender::rolling::daily("logs", "soniox.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        (BoxMakeWriter::new(non_blocking), Some(guard))
    } else {
        (BoxMakeWriter::new(std::io::stdout), None)
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(writer)
        .with_ansi(!log_to_file)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    guard
}
