use serde::{Deserialize, Serialize};
use tracing::Level;

pub const LEVELS: &[TracingLevel] = &[
    TracingLevel::Error,
    TracingLevel::Warn,
    TracingLevel::Info,
    TracingLevel::Debug,
    TracingLevel::Trace,
];

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TracingLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<TracingLevel> for Level {
    fn from(level: TracingLevel) -> Self {
        match level {
            TracingLevel::Error => Self::ERROR,
            TracingLevel::Warn => Self::WARN,
            TracingLevel::Info => Self::INFO,
            TracingLevel::Debug => Self::DEBUG,
            TracingLevel::Trace => Self::TRACE,
        }
    }
}

impl std::fmt::Display for TracingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            TracingLevel::Error => "ERROR",
            TracingLevel::Warn => "WARN",
            TracingLevel::Info => "INFO",
            TracingLevel::Debug => "DEBUG",
            TracingLevel::Trace => "TRACE",
        };
        write!(f, "{}", level)
    }
}