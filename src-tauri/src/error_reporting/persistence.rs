use super::{CrashReport, ErrorReport, ErrorReporter};
use crate::error_reporting::get_os_info;
use chrono::{DateTime, Utc};
use std::io::Write;

pub fn write_error_to_file(reporter: &ErrorReporter, error: &ErrorReport) {
    let filename = format!("errors-{}.log", Utc::now().format("%Y-%m-%d"));
    let filepath = reporter.log_dir.join(filename);

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&filepath)
    {
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

pub fn write_crash_report(reporter: &ErrorReporter, crash: &CrashReport) {
    let filename = format!("crash-{}.json", crash.timestamp.format("%Y%m%d-%H%M%S"));
    let filepath = reporter.log_dir.join(filename);

    if let Ok(json) = serde_json::to_string_pretty(crash) {
        let _ = std::fs::write(&filepath, json);
    }

    let txt_filename = format!("crash-{}.txt", crash.timestamp.format("%Y%m%d-%H%M%S"));
    let txt_filepath = reporter.log_dir.join(txt_filename);

    let report = format!(
        "=== Whisprtypr Crash Report ===\n\
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
    let _ = std::fs::write(&txt_filepath, report);
}

#[allow(dead_code)]
pub fn cleanup_old_logs(reporter: &ErrorReporter, days: u32) {
    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    if let Ok(entries) = std::fs::read_dir(&reporter.log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified: DateTime<Utc> = modified.into();
                    if modified < cutoff {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}

pub fn export_logs(reporter: &ErrorReporter) -> Result<String, std::io::Error> {
    let mut output = String::new();
    output.push_str("=== Whisprtypr Error Export ===\n\n");
    output.push_str(&format!("Generated: {}\n", Utc::now()));
    output.push_str(&format!("Version: {}\n", std::env!("CARGO_PKG_VERSION")));
    output.push_str(&format!("OS: {}\n\n", get_os_info()));

    output.push_str("=== Recent Errors ===\n\n");
    for error in reporter.get_recent_errors() {
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
    let stats = reporter.get_error_stats();
    output.push_str(&format!("Total: {}\n", stats.total_errors));
    output.push_str("By Category:\n\n");
    for (cat, count) in &stats.by_category {
        output.push_str(&format!("  {}: {}\n", cat, count));
    }
    output.push_str("By Severity:\n\n");
    for (sev, count) in &stats.by_severity {
        output.push_str(&format!("  {}: {}\n", sev, count));
    }

    Ok(output)
}
