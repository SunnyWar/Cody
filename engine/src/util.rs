use std::time::SystemTime;

/// Return an ISO-8601 / RFC3339 UTC timestamp with millisecond precision.
/// Example: 2025-09-21T14:33:01.123Z
pub fn iso_stamp_ms() -> String {
    humantime::format_rfc3339_millis(SystemTime::now()).to_string()
}
