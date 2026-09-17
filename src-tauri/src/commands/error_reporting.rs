use crate::{
    error_reporting::{ErrorCategory, ErrorReporter, ErrorSeverity, ErrorStats},
    utils::{sanitize_text, MAX_EXPORT_BYTES},
    CommandError,
};
use sentry::Level as SentryLogLevel;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn report_error(
    app: AppHandle,
    category: String,
    message: String,
    severity: String,
    stack_trace: Option<String>,
    user_action: Option<String>,
    context: Option<std::collections::HashMap<String, String>>,
) -> Result<(), CommandError> {
    let severity = match severity.to_lowercase().as_str() {
        "debug" => ErrorSeverity::Debug,
        "info" => ErrorSeverity::Info,
        "warning" => ErrorSeverity::Warning,
        "error" => ErrorSeverity::Error,
        "critical" => ErrorSeverity::Critical,
        "fatal" => ErrorSeverity::Fatal,
        _ => ErrorSeverity::Error,
    };

    let category = match category.to_lowercase().as_str() {
        "transcription" => ErrorCategory::Transcription,
        "audio" => ErrorCategory::Audio,
        "model" => ErrorCategory::Model,
        "database" => ErrorCategory::Database,
        "network" => ErrorCategory::Network,
        "filesystem" => ErrorCategory::FileSystem,
        "license" => ErrorCategory::License,
        "ui" => ErrorCategory::Ui,
        "system" => ErrorCategory::System,
        "configuration" => ErrorCategory::Configuration,
        _ => ErrorCategory::Unknown,
    };

    if let Some(reporter) = ErrorReporter::global() {
        let mut report =
            crate::error_reporting::ErrorReport::new(severity, category, message.clone());

        if let Some(trace) = stack_trace {
            report = report.with_details(trace);
        }

        if let Some(action) = user_action {
            report = report.with_context("user_action", action);
        }

        if let Some(ctx) = context {
            for (key, value) in ctx {
                report = report.with_context(key, value);
            }
        }

        reporter.report(report);

        if sentry::Hub::current().client().is_some() {
            let sentry_severity = severity.to_sentry_level();
            let _ = sentry::capture_message(&message, sentry_severity);
        }

        if let Ok(app_data_dir) = app.path().app_data_dir() {
            let _ = reporter.persist_to_file(&app_data_dir);
        }
    }

    Ok(())
}

trait ToSentryLevel {
    fn to_sentry_level(&self) -> SentryLogLevel;
}

impl ToSentryLevel for ErrorSeverity {
    fn to_sentry_level(&self) -> SentryLogLevel {
        match self {
            ErrorSeverity::Debug => SentryLogLevel::Debug,
            ErrorSeverity::Info => SentryLogLevel::Info,
            ErrorSeverity::Warning => SentryLogLevel::Warning,
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                SentryLogLevel::Error
            }
        }
    }
}

#[tauri::command]
pub async fn get_error_reports(
    limit: Option<usize>,
) -> Result<Vec<crate::error_reporting::ErrorReport>, CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        Ok(reporter.get_reports(limit))
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn get_error_stats() -> Result<ErrorStats, CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        Ok(reporter.get_stats())
    } else {
        Ok(ErrorStats {
            total_errors: 0,
            by_category: std::collections::HashMap::new(),
            by_severity: std::collections::HashMap::new(),
        })
    }
}

#[tauri::command]
pub async fn export_error_reports(
    app: AppHandle,
    format: Option<String>,
) -> Result<String, CommandError> {
    let format_str = format.unwrap_or_else(|| "json".to_string());

    let content = if let Some(reporter) = ErrorReporter::global() {
        match format_str.to_lowercase().as_str() {
            "markdown" | "md" => reporter.export_to_markdown(),
            _ => reporter.export_to_json(),
        }
    } else {
        "{}".to_string()
    };

    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let reports_dir = app_data_dir.join("reports");
        std::fs::create_dir_all(&reports_dir).ok();

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = if format_str == "markdown" || format_str == "md" {
            "md"
        } else {
            "json"
        };
        let filename = format!("error_report_{}.{}", timestamp, extension);
        let filepath = reports_dir.join(&filename);

        std::fs::write(&filepath, &content)?;

        log::info!("Error report exported to: {:?}", filepath);
    }

    Ok(content)
}

#[tauri::command]
pub async fn test_sentry() -> Result<String, CommandError> {
    if sentry::Hub::current().client().is_some() {
        sentry::capture_message(
            "Sentry test from Rust backend - this is a test message",
            SentryLogLevel::Info,
        );

        if let Some(reporter) = ErrorReporter::global() {
            let report = crate::error_reporting::ErrorReport::new(
                ErrorSeverity::Info,
                ErrorCategory::System,
                "Sentry integration test".to_string(),
            )
            .with_context("test", "true");
            reporter.report(report);
        }

        Ok("Sentry is configured and test message sent! Check your Sentry dashboard.".to_string())
    } else {
        Ok("Sentry is not configured. Set SENTRY_DSN environment variable.".to_string())
    }
}

#[tauri::command]
pub async fn save_export_file(path: String, content: String) -> Result<(), CommandError> {
    let sanitized_content =
        sanitize_text(&content, MAX_EXPORT_BYTES).map_err(CommandError::PostProcessing)?;
    let export_path = crate::utils::validate_export_path(&path)
        .map_err(|e| CommandError::Io(std::io::Error::other(e)))?;

    std::fs::write(&export_path, sanitized_content)?;
    log::info!("Export saved to: {:?}", export_path);

    Ok(())
}

#[tauri::command]
pub async fn clear_error_reports() -> Result<(), CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        reporter.clear();
    }
    log::info!("Error reports cleared");
    Ok(())
}

#[tauri::command]
pub async fn load_error_reports(app: AppHandle) -> Result<usize, CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        if let Ok(app_data_dir) = app.path().app_data_dir() {
            match reporter.load_from_file(&app_data_dir) {
                Ok(count) => {
                    log::info!("Loaded {} error reports from disk", count);
                    return Ok(count);
                }
                Err(e) => {
                    log::warn!("Failed to load error reports: {}", e);
                }
            }
        }
    }
    Ok(0)
}
