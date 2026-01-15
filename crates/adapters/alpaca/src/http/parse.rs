// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2026 Andrew Crum. All rights reserved.
//  https://github.com/agcrum
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Parsing utilities for Alpaca HTTP responses.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::http::error::AlpacaHttpError;

/// Parse an ISO 8601 timestamp string to a Unix timestamp in nanoseconds.
///
/// Alpaca uses RFC3339 format timestamps (e.g., "2023-08-25T14:30:00Z").
/// This function converts them to nanoseconds since the Unix epoch.
///
/// # Arguments
///
/// * `timestamp_str` - ISO 8601 timestamp string
///
/// # Returns
///
/// Unix timestamp in nanoseconds, or error if parsing fails
///
/// # Examples
///
/// ```
/// use nautilus_alpaca::http::parse::parse_timestamp_nanos;
///
/// let nanos = parse_timestamp_nanos("2023-08-25T14:30:00Z").unwrap();
/// assert!(nanos > 0);
/// ```
pub fn parse_timestamp_nanos(timestamp_str: &str) -> Result<u64, AlpacaHttpError> {
    let dt = DateTime::parse_from_rfc3339(timestamp_str)
        .map_err(|e| AlpacaHttpError::ValidationError(format!("Invalid timestamp: {e}")))?;

    let nanos = dt.timestamp_nanos_opt().ok_or_else(|| {
        AlpacaHttpError::ValidationError(format!("Timestamp out of range: {timestamp_str}"))
    })?;

    // Ensure timestamp is positive (after Unix epoch)
    if nanos < 0 {
        return Err(AlpacaHttpError::ValidationError(format!(
            "Timestamp before Unix epoch: {timestamp_str}"
        )));
    }

    Ok(nanos as u64)
}

/// Parse an ISO 8601 timestamp string to milliseconds since Unix epoch.
///
/// Some Alpaca endpoints return millisecond precision timestamps.
///
/// # Arguments
///
/// * `timestamp_str` - ISO 8601 timestamp string
///
/// # Returns
///
/// Unix timestamp in milliseconds
pub fn parse_timestamp_millis(timestamp_str: &str) -> Result<u64, AlpacaHttpError> {
    let dt = DateTime::parse_from_rfc3339(timestamp_str)
        .map_err(|e| AlpacaHttpError::ValidationError(format!("Invalid timestamp: {e}")))?;

    Ok(dt.timestamp_millis() as u64)
}

/// Parse a decimal string to `Decimal`.
///
/// Handles various formats including scientific notation.
///
/// # Arguments
///
/// * `value` - Decimal string
///
/// # Returns
///
/// Parsed `Decimal` value
pub fn parse_decimal(value: &str) -> Result<Decimal, AlpacaHttpError> {
    // Try parsing directly first
    if let Ok(decimal) = Decimal::from_str(value) {
        return Ok(decimal);
    }

    // If direct parsing fails, check for scientific notation
    if value.contains('e') || value.contains('E') {
        let float_value = value
            .parse::<f64>()
            .map_err(|e| AlpacaHttpError::ValidationError(format!("Invalid decimal: {e}")))?;

        Decimal::try_from(float_value)
            .map_err(|e| AlpacaHttpError::ValidationError(format!("Invalid decimal: {e}")))
    } else {
        Err(AlpacaHttpError::ValidationError(format!(
            "Invalid decimal: {}",
            value
        )))
    }
}

/// Parse an optional decimal string to `Option<Decimal>`.
///
/// Returns `None` if the input is `None` or empty.
///
/// # Arguments
///
/// * `value` - Optional decimal string
///
/// # Returns
///
/// Parsed `Option<Decimal>`
pub fn parse_optional_decimal(value: Option<&str>) -> Result<Option<Decimal>, AlpacaHttpError> {
    match value {
        Some(s) if !s.is_empty() => Ok(Some(parse_decimal(s)?)),
        _ => Ok(None),
    }
}

/// Convert a timestamp string to `DateTime<Utc>`.
///
/// # Arguments
///
/// * `timestamp_str` - ISO 8601 timestamp string
///
/// # Returns
///
/// `DateTime<Utc>` object
pub fn parse_datetime_utc(timestamp_str: &str) -> Result<DateTime<Utc>, AlpacaHttpError> {
    DateTime::parse_from_rfc3339(timestamp_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| AlpacaHttpError::ValidationError(format!("Invalid datetime: {e}")))
}

/// Parse a boolean from a string.
///
/// Handles various formats: "true"/"false", "1"/"0", "yes"/"no".
///
/// # Arguments
///
/// * `value` - Boolean string
///
/// # Returns
///
/// Boolean value
pub fn parse_bool(value: &str) -> Result<bool, AlpacaHttpError> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(AlpacaHttpError::ValidationError(format!(
            "Invalid boolean: {value}"
        ))),
    }
}

/// Parse an integer from a string.
///
/// # Arguments
///
/// * `value` - Integer string
///
/// # Returns
///
/// Parsed integer
pub fn parse_i64(value: &str) -> Result<i64, AlpacaHttpError> {
    value
        .parse::<i64>()
        .map_err(|e| AlpacaHttpError::ValidationError(format!("Invalid integer: {e}")))
}

/// Parse an unsigned integer from a string.
///
/// # Arguments
///
/// * `value` - Unsigned integer string
///
/// # Returns
///
/// Parsed unsigned integer
pub fn parse_u64(value: &str) -> Result<u64, AlpacaHttpError> {
    value
        .parse::<u64>()
        .map_err(|e| AlpacaHttpError::ValidationError(format!("Invalid unsigned integer: {e}")))
}

/// Format a Unix timestamp in nanoseconds to ISO 8601 string.
///
/// # Arguments
///
/// * `nanos` - Unix timestamp in nanoseconds
///
/// # Returns
///
/// ISO 8601 formatted string
pub fn format_timestamp_nanos(nanos: u64) -> String {
    let secs = (nanos / 1_000_000_000) as i64;
    let nsecs = (nanos % 1_000_000_000) as u32;
    let dt = DateTime::from_timestamp(secs, nsecs).unwrap_or_else(|| Utc::now());
    dt.to_rfc3339()
}

/// Format a Unix timestamp in milliseconds to ISO 8601 string.
///
/// # Arguments
///
/// * `millis` - Unix timestamp in milliseconds
///
/// # Returns
///
/// ISO 8601 formatted string
pub fn format_timestamp_millis(millis: u64) -> String {
    let secs = (millis / 1000) as i64;
    let nsecs = ((millis % 1000) * 1_000_000) as u32;
    let dt = DateTime::from_timestamp(secs, nsecs).unwrap_or_else(|| Utc::now());
    dt.to_rfc3339()
}

/// Format a `DateTime<Utc>` to ISO 8601 string.
///
/// # Arguments
///
/// * `dt` - DateTime object
///
/// # Returns
///
/// ISO 8601 formatted string
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_parse_timestamp_nanos() {
        let timestamp = "2023-08-25T14:30:00Z";
        let nanos = parse_timestamp_nanos(timestamp).unwrap();
        assert_eq!(nanos, 1692973800000000000);
    }

    #[rstest]
    fn test_parse_timestamp_nanos_with_offset() {
        let timestamp = "2023-08-25T14:30:00-05:00";
        let nanos = parse_timestamp_nanos(timestamp).unwrap();
        assert!(nanos > 0);
    }

    #[rstest]
    fn test_parse_timestamp_nanos_invalid() {
        let result = parse_timestamp_nanos("not-a-timestamp");
        assert!(result.is_err());
    }

    #[rstest]
    fn test_parse_timestamp_millis() {
        let timestamp = "2023-08-25T14:30:00Z";
        let millis = parse_timestamp_millis(timestamp).unwrap();
        assert_eq!(millis, 1692973800000);
    }

    #[rstest]
    fn test_parse_decimal() {
        assert_eq!(parse_decimal("123.45").unwrap().to_string(), "123.45");
        assert_eq!(parse_decimal("0.0001").unwrap().to_string(), "0.0001");
        assert_eq!(parse_decimal("1000000").unwrap().to_string(), "1000000");
    }

    #[rstest]
    fn test_parse_decimal_scientific() {
        assert_eq!(parse_decimal("1.23e2").unwrap().to_string(), "123");
        assert_eq!(parse_decimal("1.5e-3").unwrap().to_string(), "0.0015");
    }

    #[rstest]
    fn test_parse_decimal_invalid() {
        assert!(parse_decimal("not-a-number").is_err());
    }

    #[rstest]
    fn test_parse_optional_decimal() {
        assert_eq!(
            parse_optional_decimal(Some("123.45"))
                .unwrap()
                .unwrap()
                .to_string(),
            "123.45"
        );
        assert!(parse_optional_decimal(Some("")).unwrap().is_none());
        assert!(parse_optional_decimal(None).unwrap().is_none());
    }

    #[rstest]
    fn test_parse_datetime_utc() {
        let timestamp = "2023-08-25T14:30:00Z";
        let dt = parse_datetime_utc(timestamp).unwrap();
        assert_eq!(dt.timestamp(), 1692973800);
    }

    #[rstest]
    fn test_parse_bool() {
        assert!(parse_bool("true").unwrap());
        assert!(parse_bool("TRUE").unwrap());
        assert!(parse_bool("1").unwrap());
        assert!(parse_bool("yes").unwrap());

        assert!(!parse_bool("false").unwrap());
        assert!(!parse_bool("FALSE").unwrap());
        assert!(!parse_bool("0").unwrap());
        assert!(!parse_bool("no").unwrap());

        assert!(parse_bool("maybe").is_err());
    }

    #[rstest]
    fn test_parse_i64() {
        assert_eq!(parse_i64("12345").unwrap(), 12345);
        assert_eq!(parse_i64("-12345").unwrap(), -12345);
        assert_eq!(parse_i64("0").unwrap(), 0);
        assert!(parse_i64("not-a-number").is_err());
    }

    #[rstest]
    fn test_parse_u64() {
        assert_eq!(parse_u64("12345").unwrap(), 12345);
        assert_eq!(parse_u64("0").unwrap(), 0);
        assert!(parse_u64("-12345").is_err());
        assert!(parse_u64("not-a-number").is_err());
    }

    #[rstest]
    fn test_format_timestamp_nanos() {
        let nanos = 1692973800000000000_u64;
        let formatted = format_timestamp_nanos(nanos);
        assert_eq!(formatted, "2023-08-25T14:30:00+00:00");
    }

    #[rstest]
    fn test_format_timestamp_millis() {
        let millis = 1692973800000_u64;
        let formatted = format_timestamp_millis(millis);
        assert_eq!(formatted, "2023-08-25T14:30:00+00:00");
    }

    #[rstest]
    fn test_roundtrip_timestamp_nanos() {
        let original = "2023-08-25T14:30:00Z";
        let nanos = parse_timestamp_nanos(original).unwrap();
        let formatted = format_timestamp_nanos(nanos);
        let reparsed = parse_timestamp_nanos(&formatted).unwrap();
        assert_eq!(nanos, reparsed);
    }

    #[rstest]
    fn test_format_datetime() {
        let dt = DateTime::parse_from_rfc3339("2023-08-25T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = format_datetime(&dt);
        assert_eq!(formatted, "2023-08-25T14:30:00+00:00");
    }
}
