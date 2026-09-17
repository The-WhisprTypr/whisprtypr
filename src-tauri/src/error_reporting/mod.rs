pub mod exports;
pub mod models;
pub mod persistence;

pub use exports::{get_os_info, report_critical_error, report_error, report_error_with_details};
pub use models::{CrashReport, ErrorCategory, ErrorReport, ErrorSeverity, ErrorStats};

use chrono::Utc;
use log::{error, info, warn};
use std::fs;
use std::panic;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

static ERROR_REPORTER: OnceLock<Arc<ErrorReporter>> = OnceLock::new();

pub struct ErrorReporter {
    pub(crate) log_dir: PathBuf,
    pub(crate) recent_errors: Mutex<Vec<ErrorReport>>,
    pub(crate) error_counts: Mutex<HashMap<String, u32>>,
    pub(crate) max_recent_errors: usize,
    #[allow(dead_code)]
    pub(crate) telemetry_enabled: bool,
}

use std::collections::HashMap;

impl ErrorReporter {
    pub fn new(log_dir: PathBuf) -> Self {
        fs::create_dir_all(&log_dir).ok();

        Self {
            log_dir,
            recent_errors: Mutex::new(Vec::new()),
            error_counts: Mutex::new(HashMap::new()),
            max_recent_errors: 100,
            telemetry_enabled: false,
        }
    }

    pub fn init(log_dir: PathBuf) {
        let _ = ERROR_REPORTER.get_or_init(|| {
            let reporter = Arc::new(ErrorReporter::new(log_dir));

            let reporter_clone = reporter.clone();
            panic::set_hook(Box::new(move |info| {
                reporter_clone.handle_panic(info);
            }));

            info!("Error reporter initialized");
            reporter
        });
    }

    pub fn global() -> Option<Arc<ErrorReporter>> {
        ERROR_REPORTER.get().cloned()
    }

    pub fn report(&self, error: ErrorReport) {
        let fingerprint = error.fingerprint();

        let occurrence_count = {
            let mut counts = self.error_counts.lock().unwrap();
            let count = counts.entry(fingerprint.clone()).or_insert(0);
            *count += 1;
            *count
        };

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

        if occurrence_count <= 10 || occurrence_count % 100 == 0 {
            let mut error_with_count = error.clone();
            error_with_count.occurrence_count = occurrence_count;

            let mut recent = self.recent_errors.lock().unwrap();
            recent.push(error_with_count.clone());

            if recent.len() > self.max_recent_errors {
                recent.remove(0);
            }

            if error.severity as u8 >= ErrorSeverity::Error as u8 {
                persistence::write_error_to_file(self, &error_with_count);
            }
        }
    }

    fn handle_panic(&self, info: &std::panic::PanicHookInfo<'_>) {
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
            backtrace: format!("{:?}", std::backtrace::Backtrace::force_capture()),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_info: get_os_info(),
            thread_name: std::thread::current().name().map(String::from),
        };

        error!("=== CRASH DETECTED ===");
        error!("Message: {}", panic_message);
        if let Some(loc) = location {
            error!("Location: {}", loc);
        }
        error!("Thread: {:?}", crash_report.thread_name);

        persistence::write_crash_report(self, &crash_report);
    }

    pub fn get_recent_errors(&self) -> Vec<ErrorReport> {
        self.recent_errors.lock().unwrap().clone()
    }

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

    pub fn get_reports(&self, limit: Option<usize>) -> Vec<ErrorReport> {
        let errors = self.recent_errors.lock().unwrap();
        match limit {
            Some(n) => errors.iter().rev().take(n).cloned().collect(),
            None => errors.clone(),
        }
    }

    pub fn get_stats(&self) -> ErrorStats {
        self.get_error_stats()
    }

    pub fn clear(&self) {
        let mut errors = self.recent_errors.lock().unwrap();
        errors.clear();
        let mut counts = self.error_counts.lock().unwrap();
        counts.clear();
        info!("Error reports cleared from memory");
    }

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

    pub fn export_to_markdown(&self) -> String {
        let errors = self.get_recent_errors();
        let stats = self.get_error_stats();

        let mut md = String::new();
        md.push_str("# Whisprtypr Error Report\n\n");
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

    pub fn persist_to_file(&self, app_dir: &std::path::Path) -> Result<(), std::io::Error> {
        let errors_dir = app_dir.join("errors");
        fs::create_dir_all(&errors_dir)?;

        let filepath = errors_dir.join("errors.json");
        let errors = self.get_recent_errors();
        let json = serde_json::to_string_pretty(&errors)?;
        fs::write(filepath, json)?;
        Ok(())
    }

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

        while errors.len() > self.max_recent_errors {
            errors.remove(0);
        }

        Ok(count)
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
        .with_context("url", "https://github.com/The-Whisprtypr/whisprtypr");

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
