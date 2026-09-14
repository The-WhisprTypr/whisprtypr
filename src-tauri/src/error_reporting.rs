//! Production-grade error reporting and crash handling
//!
//! Provides:
//! - Structured error logging with context
//! - Error aggregation and deduplication
//! - Crash report generation
//! - Local error history for debugging
//! - Optional telemetry hooks (disabled by default for privacy)

use chrono::{DateTime, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic::{self, PanicHookInfo};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

static ERROR_REPORTER: OnceLock<Arc<ErrorReporter>> = OnceLock::new();

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    /// Debug-level issues, not shown to users
    Debug,
    /// Informational, operation succeeded with notes
    Info,
    /// Warning, operation succeeded but with issues
    Warning,
    /// Error, operation failed but app can continue
    Error,
    /// Critical, operation failed and may affect app stability
    Critical,
    /// Fatal, app cannot continue
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

/// Error categories for grouping and analysis
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

/// Structured error report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    /// Unique error ID for tracking
    pub id: String,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Error message
    pub message: String,
    /// Technical details (not shown to users)
    pub details: Option<String>,
    /// Stack trace if available
    pub backtrace: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
    /// Number of times this error occurred
    pub occurrence_count: u32,
    /// App version
    pub app_version: String,
    /// OS information
    pub os_info: String,
}

impl ErrorReport {
    /// Create a new error report
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
            os_info: get_os_info(),
        }
    }

    /// Add technical details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add backtrace
    #[allow(dead_code)]
    pub fn with_backtrace(mut self) -> Self {
        self.backtrace = Some(format!("{:?}", Backtrace::capture()));
        self
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Get a fingerprint for deduplication
    pub fn fingerprint(&self) -> String {
        format!("{}:{}:{}", self.category, self.severity, self.message)
    }
}

/// Crash report for unhandled panics
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

/// Error reporter with aggregation and persistence
pub struct ErrorReporter {
    /// Directory for storing error logs
    log_dir: PathBuf,
    /// Recent errors (in-memory cache)
    recent_errors: Mutex<Vec<ErrorReport>>,
    /// Error counts by fingerprint (for deduplication)
    error_counts: Mutex<HashMap<String, u32>>,
    /// Maximum errors to keep in memory
    max_recent_errors: usize,
    /// Whether telemetry is enabled (reserved for future use)
    #[allow(dead_code)]
    telemetry_enabled: bool,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new(log_dir: PathBuf) -> Self {
        // Ensure log directory exists
        fs::create_dir_all(&log_dir).ok();

        Self {
            log_dir,
            recent_errors: Mutex::new(Vec::new()),
            error_counts: Mutex::new(HashMap::new()),
            max_recent_errors: 100,
            telemetry_enabled: false, // Disabled by default for privacy
        }
    }

    /// Initialize the global error reporter
    pub fn init(log_dir: PathBuf) {
        let _ = ERROR_REPORTER.get_or_init(|| {
            let reporter = Arc::new(ErrorReporter::new(log_dir));

            // Set up panic hook for crash reporting
            let reporter_clone = reporter.clone();
            panic::set_hook(Box::new(move |info| {
                reporter_clone.handle_panic(info);
            }));

            info!("Error reporter initialized");
            reporter
        });
    }

    /// Get the global error reporter
    pub fn global() -> Option<Arc<ErrorReporter>> {
        ERROR_REPORTER.get().cloned()
    }

    /// Report an error
    pub fn report(&self, error: ErrorReport) {
        let fingerprint = error.fingerprint();

        // Update occurrence count
        let occurrence_count = {
            let mut counts = self.error_counts.lock().unwrap();
            let count = counts.entry(fingerprint.clone()).or_insert(0);
            *count += 1;
            *count
        };

        // Log based on severity
        match error.severity {
            ErrorSeverity::Debug => {
                log::debug!("[{}] {}: {}", error.category, error.severity, error.message);
            }
            ErrorSeverity::Info => {
                info!("[{}] {}", error.category, error.message);
            }
            ErrorSeverity::Warning => {
                warn!("[{}] {}", error.category, error.message);
            }
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                error!("[{}] {}: {}", error.category, error.severity, error.message);
                if let Some(ref details) = error.details {
                    error!("  Details: {}", details);
                }
            }
        }

        // Only store unique errors or rate-limit duplicates
        if occurrence_count <= 10 || occurrence_count % 100 == 0 {
            let mut error_with_count = error.clone();
            error_with_count.occurrence_count = occurrence_count;

            // Add to recent errors
            let mut recent = self.recent_errors.lock().unwrap();
            recent.push(error_with_count.clone());

            // Trim if too many
            if recent.len() > self.max_recent_errors {
                recent.remove(0);
            }

            // Write to log file for critical+ errors
            if error.severity as u8 >= ErrorSeverity::Error as u8 {
                self.write_error_to_file(&error_with_count);
            }
        }
    }

    /// Handle a panic/crash
    fn handle_panic(&self, info: &PanicHookInfo<'_>) {
        let panic_message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));

        let crash_report = CrashReport {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            panic_message: panic_message.clone(),
            backtrace: format!("{:?}", Backtrace::force_capture()),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_info: get_os_info(),
            thread_name: std::thread::current().name().map(String::from),
        };

        // Log the crash
        error!("=== CRASH DETECTED ===");
        error!("Message: {}", panic_message);
        if let Some(loc) = location {
            error!("Location: {}", loc);
        }
        error!("Thread: {:?}", crash_report.thread_name);

        // Write crash report to file
        self.write_crash_report(&crash_report);
    }

    /// Write error to log file
    fn write_error_to_file(&self, error: &ErrorReport) {
        let filename = format!("errors-{}.log", Utc::now().format("%Y-%m-%d"));
        let filepath = self.log_dir.join(filename);

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&filepath) {
            let log_line = format!(
                "[{}] {} | {} | {} | {}\n",
                error.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                error.severity,
                error.category,
                error.message,
                error.details.as_deref().unwrap_or("")
            );
            let _ = file.write_all(log_line.as_bytes());
        }
    }

    /// Write crash report to file
    fn write_crash_report(&self, crash: &CrashReport) {
        let filename = format!("crash-{}.json", crash.timestamp.format("%Y%m%d-%H%M%S"));
        let filepath = self.log_dir.join(filename);

        if let Ok(json) = serde_json::to_string_pretty(crash) {
            let _ = fs::write(&filepath, json);
        }

        // Also write a human-readable version
        let txt_filename = format!("crash-{}.txt", crash.timestamp.format("%Y%m%d-%H%M%S"));
        let txt_filepath = self.log_dir.join(txt_filename);

        let report = format!(
            "=== WhisprTypr Crash Report ===\n\
            Time: {}\n\
            Version: {}\n\
            OS: {}\n\
            Thread: {:?}\n\n\
            Error: {}\n\n\
            Backtrace:\n{}\n",
            crash.timestamp,
            crash.app_version,
            crash.os_info,
            crash.thread_name,
            crash.panic_message,
            crash.backtrace
        );
        let _ = fs::write(&txt_filepath, report);
    }

    /// Get recent errors
    pub fn get_recent_errors(&self) -> Vec<ErrorReport> {
        self.recent_errors.lock().unwrap().clone()
    }

    /// Get error statistics
    pub fn get_error_stats(&self) -> ErrorStats {
        let errors = self.recent_errors.lock().unwrap();
        let mut by_category: HashMap<String, u32> = HashMap::new();
        let mut by_severity: HashMap<String, u32> = HashMap::new();

        for error in errors.iter() {
            *by_category.entry(error.category.to_string()).or_insert(0) += 1;
            *by_severity.entry(error.severity.to_string()).or_insert(0) += 1;
        }

        ErrorStats {
            total_errors: errors.len() as u32,
            by_category,
            by_severity,
        }
    }

    /// Clear old log files (older than days)
    #[allow(dead_code)]
    pub fn cleanup_old_logs(&self, days: u32) {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified: DateTime<Utc> = modified.into();
                        if modified < cutoff {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }

    /// Export error logs for support
    #[allow(dead_code)]
    pub fn export_logs(&self) -> Result<String, std::io::Error> {
        let mut output = String::new();
        output.push_str("=== WhisprTypr Error Export ===\n\n");
        output.push_str(&format!("Generated: {}\n", Utc::now()));
        output.push_str(&format!("Version: {}\n", env!("CARGO_PKG_VERSION")));
        output.push_str(&format!("OS: {}\n\n", get_os_info()));

        output.push_str("=== Recent Errors ===\n\n");
        for error in self.get_recent_errors() {
            output.push_str(&format!(
                "[{}] {} | {} | {}\n",
                error.timestamp.format("%Y-%m-%d %H:%M:%S"),
                error.severity,
                error.category,
                error.message
            ));
            if let Some(details) = &error.details {
                output.push_str(&format!("  Details: {}\n", details));
            }
            output.push('\n');
        }

        output.push_str("=== Error Statistics ===\n\n");
        let stats = self.get_error_stats();
        output.push_str(&format!("Total: {}\n", stats.total_errors));
        output.push_str("By Category:\n");
        for (cat, count) in &stats.by_category {
            output.push_str(&format!("  {}: {}\n", cat, count));
        }
        output.push_str("By Severity:\n");
        for (sev, count) in &stats.by_severity {
            output.push_str(&format!("  {}: {}\n", sev, count));
        }

        Ok(output)
    }

    /// Get recent errors with optional limit
    pub fn get_reports(&self, limit: Option<usize>) -> Vec<ErrorReport> {
        let errors = self.recent_errors.lock().unwrap();
        match limit {
            Some(n) => errors.iter().rev().take(n).cloned().collect(),
            None => errors.clone(),
        }
    }

    /// Get stats (alias for get_error_stats for lib.rs)
    pub fn get_stats(&self) -> ErrorStats {
        self.get_error_stats()
    }

    /// Export to JSON format
    pub fn export_to_json(&self) -> String {
        let errors = self.get_recent_errors();
        let stats = self.get_error_stats();
        let export = serde_json::json!({
            "generated_at": Utc::now().to_rfc3339(),
            "app_version": env!("CARGO_PKG_VERSION"),
            "os_info": get_os_info(),
            "errors": errors,
            "stats": stats,
        });
        serde_json::to_string_pretty(&export).unwrap_or_else(|_| "{}".to_string())
    }

    /// Export to Markdown format
    pub fn export_to_markdown(&self) -> String {
        let errors = self.get_recent_errors();
        let stats = self.get_error_stats();

        let mut md = String::new();
        md.push_str("# WhisprTypr Error Report\n\n");
        md.push_str(&format!(
            "**Generated:** {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        md.push_str(&format!("**Version:** {}\n\n", env!("CARGO_PKG_VERSION")));
        md.push_str(&format!("**OS:** {}\n\n", get_os_info()));

        md.push_str("## Statistics\n\n");
        md.push_str(&format!("- **Total Errors:** {}\n\n", stats.total_errors));

        md.push_str("### By Category\n\n");
        for (cat, count) in &stats.by_category {
            md.push_str(&format!("- {}: {}\n", cat, count));
        }
        md.push_str("\n### By Severity\n\n");
        for (sev, count) in &stats.by_severity {
            md.push_str(&format!("- {}: {}\n", sev, count));
        }

        md.push_str("\n## Recent Errors\n\n");
        for error in errors.iter().rev().take(50) {
            md.push_str(&format!(
                "### {} - {}\n\n**Time:** {}\n**Category:** {}\n**Message:** {}\n",
                error.severity,
                error.id,
                error.timestamp.format("%Y-%m-%d %H:%M:%S"),
                error.category,
                error.message
            ));
            if let Some(details) = &error.details {
                md.push_str(&format!("**Details:** {}\n", details));
            }
            md.push_str("\n---\n\n");
        }

        md
    }

    /// Clear all errors from memory
    pub fn clear(&self) {
        let mut errors = self.recent_errors.lock().unwrap();
        errors.clear();
        let mut counts = self.error_counts.lock().unwrap();
        counts.clear();
        info!("Error reports cleared from memory");
    }

    /// Persist errors to a file
    pub fn persist_to_file(&self, app_dir: &std::path::Path) -> Result<(), std::io::Error> {
        let errors_dir = app_dir.join("errors");
        fs::create_dir_all(&errors_dir)?;

        let filepath = errors_dir.join("errors.json");
        let errors = self.get_recent_errors();
        let json = serde_json::to_string_pretty(&errors)?;
        fs::write(filepath, json)?;
        Ok(())
    }

    /// Load errors from file
    pub fn load_from_file(&self, app_dir: &std::path::Path) -> Result<usize, std::io::Error> {
        let filepath = app_dir.join("errors").join("errors.json");
        if !filepath.exists() {
            return Ok(0);
        }

        let json = fs::read_to_string(filepath)?;
        let loaded_errors: Vec<ErrorReport> = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let count = loaded_errors.len();
        let mut errors = self.recent_errors.lock().unwrap();
        errors.extend(loaded_errors);

        // Trim to max
        while errors.len() > self.max_recent_errors {
            errors.remove(0);
        }

        Ok(count)
    }
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: u32,
    pub by_category: HashMap<String, u32>,
    pub by_severity: HashMap<String, u32>,
}

/// Get OS information
fn get_os_info() -> String {
    format!(
        "{} {} ({})",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::consts::FAMILY
    )
}

// ============================================
// Convenience macros and functions
// ============================================

/// Report an error using the global reporter
#[allow(dead_code)]
pub fn report_error(severity: ErrorSeverity, category: ErrorCategory, message: impl Into<String>) {
    if let Some(reporter) = ErrorReporter::global() {
        reporter.report(ErrorReport::new(severity, category, message));
    }
}

/// Report an error with details
#[allow(dead_code)]
pub fn report_error_with_details(
    severity: ErrorSeverity,
    category: ErrorCategory,
    message: impl Into<String>,
    details: impl Into<String>,
) {
    if let Some(reporter) = ErrorReporter::global() {
        reporter.report(ErrorReport::new(severity, category, message).with_details(details));
    }
}

/// Report a critical error with backtrace
#[allow(dead_code)]
pub fn report_critical_error(
    category: ErrorCategory,
    message: impl Into<String>,
    details: impl Into<String>,
) {
    if let Some(reporter) = ErrorReporter::global() {
        reporter.report(
            ErrorReport::new(ErrorSeverity::Critical, category, message)
                .with_details(details)
                .with_backtrace(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_error_report_creation() {
        let report = ErrorReport::new(ErrorSeverity::Error, ErrorCategory::Audio, "Test error");

        assert_eq!(report.severity, ErrorSeverity::Error);
        assert_eq!(report.category, ErrorCategory::Audio);
        assert_eq!(report.message, "Test error");
    }

    #[test]
    fn test_error_report_with_context() {
        let report = ErrorReport::new(
            ErrorSeverity::Warning,
            ErrorCategory::Network,
            "Connection failed",
        )
        .with_details("Timeout after 30s")
        .with_context("url", "https://github.com/johuniq/whisprtypr");

        assert!(report.details.is_some());
        assert!(report.context.contains_key("url"));
    }

    #[test]
    fn test_error_reporter() {
        let dir = tempdir().unwrap();
        let reporter = ErrorReporter::new(dir.path().to_path_buf());

        reporter.report(ErrorReport::new(
            ErrorSeverity::Error,
            ErrorCategory::Database,
            "Test error",
        ));

        let recent = reporter.get_recent_errors();
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_error_stats() {
        let dir = tempdir().unwrap();
        let reporter = ErrorReporter::new(dir.path().to_path_buf());

        reporter.report(ErrorReport::new(
            ErrorSeverity::Error,
            ErrorCategory::Audio,
            "Error 1",
        ));
        reporter.report(ErrorReport::new(
            ErrorSeverity::Warning,
            ErrorCategory::Audio,
            "Warning 1",
        ));
        reporter.report(ErrorReport::new(
            ErrorSeverity::Error,
            ErrorCategory::Network,
            "Error 2",
        ));

        let stats = reporter.get_error_stats();
        assert_eq!(stats.total_errors, 3);
    }

    #[test]
    fn test_fingerprint_deduplication() {
        let error1 = ErrorReport::new(ErrorSeverity::Error, ErrorCategory::Audio, "Same error");
        let error2 = ErrorReport::new(ErrorSeverity::Error, ErrorCategory::Audio, "Same error");

        assert_eq!(error1.fingerprint(), error2.fingerprint());
    }
}
