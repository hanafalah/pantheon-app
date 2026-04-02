//! Date/Time Helper Functions

use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};

/// Get current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Get current timestamp in seconds
pub fn timestamp() -> i64 {
    Utc::now().timestamp()
}

/// Get current timestamp in milliseconds
pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// Parse date from string (YYYY-MM-DD format)
pub fn parse_date(s: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

/// Format date to YYYY-MM-DD string
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Format datetime to ISO 8601 string
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Add days to datetime
pub fn add_days(dt: &DateTime<Utc>, days: i64) -> DateTime<Utc> {
    *dt + Duration::days(days)
}

/// Add hours to datetime
pub fn add_hours(dt: &DateTime<Utc>, hours: i64) -> DateTime<Utc> {
    *dt + Duration::hours(hours)
}

/// Add minutes to datetime
pub fn add_minutes(dt: &DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
    *dt + Duration::minutes(minutes)
}

/// Subtract days from datetime
pub fn sub_days(dt: &DateTime<Utc>, days: i64) -> DateTime<Utc> {
    *dt - Duration::days(days)
}

/// Get difference in days between two datetimes
pub fn diff_days(dt1: &DateTime<Utc>, dt2: &DateTime<Utc>) -> i64 {
    (*dt1 - *dt2).num_days()
}

/// Get difference in hours between two datetimes
pub fn diff_hours(dt1: &DateTime<Utc>, dt2: &DateTime<Utc>) -> i64 {
    (*dt1 - *dt2).num_hours()
}

/// Check if datetime is in the past
pub fn is_past(dt: &DateTime<Utc>) -> bool {
    *dt < Utc::now()
}

/// Check if datetime is in the future
pub fn is_future(dt: &DateTime<Utc>) -> bool {
    *dt > Utc::now()
}

/// Get start of day
pub fn start_of_day(dt: &DateTime<Utc>) -> DateTime<Utc> {
    dt.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()
}

/// Get end of day
pub fn end_of_day(dt: &DateTime<Utc>) -> DateTime<Utc> {
    dt.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_now() {
        let dt = now();
        assert!(dt.timestamp() > 0);
    }

    #[test]
    fn test_timestamp() {
        let ts = timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2024-01-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(format_date(&date), "2024-01-15");
    }

    #[test]
    fn test_add_days() {
        let dt = Utc::now();
        let future = add_days(&dt, 5);
        assert!(diff_days(&future, &dt) == 5);
    }

    #[test]
    fn test_sub_days() {
        let dt = Utc::now();
        let past = sub_days(&dt, 5);
        assert!(diff_days(&dt, &past) == 5);
    }

    #[test]
    fn test_is_past_future() {
        let past = sub_days(&Utc::now(), 1);
        let future = add_days(&Utc::now(), 1);

        assert!(is_past(&past));
        assert!(is_future(&future));
    }

    #[test]
    fn test_start_end_of_day() {
        let dt = Utc::now();
        let start = start_of_day(&dt);
        let end = end_of_day(&dt);

        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
    }
}
