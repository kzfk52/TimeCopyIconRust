// Date/time conversion logic ported from the original WinForms app's
// Helper.cs / Form1.cs. Timezone is taken as a generic parameter so tests
// can pin a fixed offset instead of depending on the host's local timezone.

use chrono::{DateTime, Local, NaiveDateTime, Offset, TimeZone};

/// Mirrors `Helper.getUnixTimeStr(DateTimeOffset.Now)`.
pub fn unixtime_str_now() -> String {
    chrono::Utc::now().timestamp().to_string()
}

/// Mirrors `Helper.getYmd1Str`: `yyyy/MM/dd HH:mm:ss` in the given timezone.
pub fn ymd1_str_now<Tz: TimeZone>(tz: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    tz.from_utc_datetime(&chrono::Utc::now().naive_utc())
        .format("%Y/%m/%d %H:%M:%S")
        .to_string()
}

/// Mirrors `Helper.getYmd2Str`: `yyyyMMddHHmmss` in the given timezone.
pub fn ymd2_str_now<Tz: TimeZone>(tz: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    tz.from_utc_datetime(&chrono::Utc::now().naive_utc())
        .format("%Y%m%d%H%M%S")
        .to_string()
}

/// Renders a unix-seconds timestamp as `yyyy/MM/dd HH:mm:ss` in the given timezone.
fn local_string_from_unix_seconds<Tz: TimeZone>(unix_seconds: i64, tz: &Tz) -> Option<String>
where
    Tz::Offset: std::fmt::Display,
{
    let utc_dt = DateTime::from_timestamp(unix_seconds, 0)?;
    Some(
        tz.from_utc_datetime(&utc_dt.naive_utc())
            .format("%Y/%m/%d %H:%M:%S")
            .to_string(),
    )
}

/// Mirrors `Form1.textBoxFromUnixtime_Leave` / `UnixTimeToLocalDateString` /
/// `UnixMicroTimeToLocalDateString`.
///
/// The original C# decides seconds-vs-milliseconds purely by the *string
/// length* of the trimmed input (`txt.Length == 13`), not by the numeric
/// magnitude. That quirk is intentionally preserved here.
///
/// Unlike the original, which lets `DateTimeOffset.FromUnixTimeSeconds`
/// throw on out-of-range values (an unhandled exception in the C# app),
/// this returns `None` for out-of-range input so the caller can safely
/// clear the output field instead of crashing.
pub fn unixtime_input_to_local_string<Tz: TimeZone>(input: &str, tz: &Tz) -> Option<String>
where
    Tz::Offset: std::fmt::Display,
{
    let trimmed = input.trim();
    let value: i64 = trimmed.parse().ok()?;

    let unix_seconds = if trimmed.len() == 13 {
        value.div_euclid(1000)
    } else {
        value
    };

    local_string_from_unix_seconds(unix_seconds, tz)
}

/// Mirrors `Form1.textBox4_Leave`'s use of `DateTimeOffset.TryParse`, which
/// accepts a wide range of date/time string shapes. `chrono` has no single
/// equivalent, so candidate formats are tried in order, most specific first.
///
/// `local_tz` is used to interpret strings that carry no explicit UTC
/// offset, exactly as `DateTimeOffset.TryParse` falls back to the system's
/// local timezone in the original app. It is a generic parameter (rather
/// than reaching for `chrono::Local` directly) purely so unit tests can
/// pin a fixed offset and stay deterministic regardless of the host's
/// timezone; production code should pass `chrono::Local`.
pub fn parse_flexible_datetime_to_unix<Tz: TimeZone>(input: &str, local_tz: &Tz) -> Option<i64> {
    let s = input.trim();

    // e.g. "2026-09-08T10:32:14+0900" (the format the sample field itself generates).
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z") {
        return Some(dt.timestamp());
    }
    // e.g. "2026-09-08T10:32:14+09:00" (RFC 3339 / colon-separated offset ISO 8601).
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp());
    }

    for fmt in [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
    ] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
            if let Some(local) = local_tz.from_local_datetime(&naive).single() {
                return Some(local.timestamp());
            }
        }
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let naive = date.and_hms_opt(0, 0, 0)?;
        if let Some(local) = local_tz.from_local_datetime(&naive).single() {
            return Some(local.timestamp());
        }
    }

    None
}

/// Mirrors `Form1_Load`'s construction of the ISO 8601 sample text:
/// `now.ToString("yyyy-MM-ddTHH:mm:ss") + now.ToString("zzz").Replace(":", "")`
/// i.e. an offset with no colon, e.g. `2026-09-08T10:32:14+0900`.
pub fn iso8601_example_now() -> String {
    let now = Local::now();
    let offset_str = now.offset().fix().to_string().replace(':', "");
    format!("{}{}", now.format("%Y-%m-%dT%H:%M:%S"), offset_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn jst() -> FixedOffset {
        FixedOffset::east_opt(9 * 3600).unwrap()
    }

    #[test]
    fn unixtime_seconds_to_local_string_matches_screenshot_example() {
        // From the reference screenshot: 1787818376 -> 2026/08/27 17:12:56 (JST).
        let result = unixtime_input_to_local_string("1787818376", &jst());
        assert_eq!(result.as_deref(), Some("2026/08/27 17:12:56"));
    }

    #[test]
    fn thirteen_digit_input_is_treated_as_milliseconds() {
        // 1787818376000 ms == 1787818376 s -> same local time as above.
        let result = unixtime_input_to_local_string("1787818376000", &jst());
        assert_eq!(result.as_deref(), Some("2026/08/27 17:12:56"));
    }

    #[test]
    fn non_numeric_input_returns_none() {
        assert_eq!(unixtime_input_to_local_string("not-a-number", &jst()), None);
    }

    #[test]
    fn out_of_range_input_returns_none_instead_of_panicking() {
        assert_eq!(unixtime_input_to_local_string("99999999999999999", &jst()), None);
    }

    #[test]
    fn parses_sample_iso8601_format_with_colonless_offset() {
        // Matches what iso8601_example_now() itself generates.
        let unix = parse_flexible_datetime_to_unix("2026-09-08T10:32:14+0900", &jst()).unwrap();
        let back = local_string_from_unix_seconds(unix, &jst()).unwrap();
        assert_eq!(back, "2026/09/08 10:32:14");
    }

    #[test]
    fn parses_rfc3339_with_colon_offset() {
        let unix = parse_flexible_datetime_to_unix("2026-09-08T10:32:14+09:00", &jst()).unwrap();
        let back = local_string_from_unix_seconds(unix, &jst()).unwrap();
        assert_eq!(back, "2026/09/08 10:32:14");
    }

    #[test]
    fn parses_space_separated_date_matches_screenshot_example() {
        // From the reference screenshot: "2026-08-24 17:42:23" -> 1787560943 (interpreted in JST).
        let unix = parse_flexible_datetime_to_unix("2026-08-24 17:42:23", &jst()).unwrap();
        assert_eq!(unix, 1787560943);
    }

    #[test]
    fn unparseable_input_returns_none() {
        assert_eq!(parse_flexible_datetime_to_unix("not a date", &jst()), None);
    }
}
