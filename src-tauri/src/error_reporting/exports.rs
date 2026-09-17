use crate::error_reporting::{ErrorCategory, ErrorReport, ErrorReporter, ErrorSeverity};

pub fn report_error(severity: ErrorSeverity, category: ErrorCategory, message: impl Into<String>) {
    if let Some(reporter) = ErrorReporter::global() {
        reporter.report(ErrorReport::new(severity, category, message));
    }
}

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

pub fn get_os_info() -> String {
    format!(
        "{} {} ({})",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::consts::FAMILY
    )
}
