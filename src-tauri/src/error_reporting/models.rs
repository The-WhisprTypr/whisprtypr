use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
    Fatal,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Debug => write!(f, "DEBUG"),
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARN"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Audio,
    Transcription,
    Model,
    Database,
    Network,
    FileSystem,
    Hotkey,
    TextInjection,
    License,
    Ui,
    System,
    Configuration,
    Unknown,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Audio => write!(f, "audio"),
            ErrorCategory::Transcription => write!(f, "transcription"),
            ErrorCategory::Model => write!(f, "model"),
            ErrorCategory::Database => write!(f, "database"),
            ErrorCategory::Network => write!(f, "network"),
            ErrorCategory::FileSystem => write!(f, "filesystem"),
            ErrorCategory::Hotkey => write!(f, "hotkey"),
            ErrorCategory::TextInjection => write!(f, "text_injection"),
            ErrorCategory::License => write!(f, "license"),
            ErrorCategory::Ui => write!(f, "ui"),
            ErrorCategory::System => write!(f, "system"),
            ErrorCategory::Configuration => write!(f, "configuration"),
            ErrorCategory::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: ErrorSeverity,
    pub category: ErrorCategory,
    pub message: String,
    pub details: Option<String>,
    pub backtrace: Option<String>,
    pub context: HashMap<String, String>,
    pub occurrence_count: u32,
    pub app_version: String,
    pub os_info: String,
}

impl ErrorReport {
    pub fn new(
        severity: ErrorSeverity,
        category: ErrorCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            severity,
            category,
            message: message.into(),
            details: None,
            backtrace: None,
            context: HashMap::new(),
            occurrence_count: 1,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_info: super::get_os_info(),
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_backtrace(mut self) -> Self {
        self.backtrace = Some(format!("{:?}", Backtrace::capture()));
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn fingerprint(&self) -> String {
        format!("{}:{}:{}", self.category, self.severity, self.message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub panic_message: String,
    pub backtrace: String,
    pub app_version: String,
    pub os_info: String,
    pub thread_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: u32,
    pub by_category: HashMap<String, u32>,
    pub by_severity: HashMap<String, u32>,
}
