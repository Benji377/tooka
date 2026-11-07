//! File matching utilities for Tooka.
//!
//! This module provides functions to match files against various criteria,
//! including filename patterns, extensions, paths, sizes, MIME types, dates,
//! symlink status, EXIF metadata, and combined rule conditions.

use crate::{
    core::error::TookaError,
    rules::rule::{self, Conditions, DateRange, Range},
    utils::date_parser::parse_date,
};

/// The plain text of an EXIF text (ASCII) value, without the quotes and
/// escaping that `display_value` adds. `None` for other value types.
pub(crate) fn exif_text(value: &exif::Value) -> Option<String> {
    match value {
        exif::Value::Ascii(parts) => Some(
            parts
                .iter()
                .map(|p| {
                    String::from_utf8_lossy(p)
                        .trim_end_matches('\0')
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        _ => None,
    }
}

use chrono::{NaiveDate, Utc};
use exif::Reader;
use glob::{self, Pattern};
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::sync::LazyLock;

const MIN_DATE: (i32, u32, u32) = (1970, 1, 1);
const MAX_DATE: (i32, u32, u32) = (9999, 12, 31);

/// Cached minimum date for range comparisons
static MIN_DATE_NAIVE: LazyLock<NaiveDate> = LazyLock::new(|| {
    NaiveDate::from_ymd_opt(MIN_DATE.0, MIN_DATE.1, MIN_DATE.2).expect("MIN_DATE should be valid")
});

/// Cached maximum date for range comparisons
static MAX_DATE_NAIVE: LazyLock<NaiveDate> = LazyLock::new(|| {
    NaiveDate::from_ymd_opt(MAX_DATE.0, MAX_DATE.1, MAX_DATE.2).expect("MAX_DATE should be valid")
});

/// Matches a file's name against a regular expression pattern
pub(crate) fn match_filename_regex(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    log::debug!(
        "Matching file: {} against pattern: {}",
        file_path.display(),
        pattern
    );
    let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let regex = regex::Regex::new(pattern)?;
    Ok(regex.is_match(file_name))
}

/// Matches a file against a given vector of file extensions
pub(crate) fn match_extensions(file_path: &Path, extensions: &[String]) -> bool {
    log::debug!(
        "Matching file: {} against extensions: {:?}",
        file_path.display(),
        extensions
    );
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext_str| extensions.iter().any(|ext| ext.as_str() == ext_str))
}

/// Matches a file path against a glob pattern
pub(crate) fn match_path(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    log::debug!(
        "Matching file: {} against glob pattern: {}",
        file_path.display(),
        pattern
    );
    let file_path_str = file_path.to_string_lossy();
    let glob_pattern = glob::Pattern::new(pattern)?;
    Ok(glob_pattern.matches(&file_path_str))
}

/// Matches a file's size against a given size range in kilobytes
pub(crate) fn match_size_kb(metadata: &fs::Metadata, size_kb: &Range) -> bool {
    log::debug!(
        "Matching file size: {} against range: {:?}",
        metadata.len(),
        size_kb
    );
    let size = metadata.len();
    let min = match size_kb.min {
        Some(m) => m.saturating_mul(1024),
        None => 0,
    };
    let max = match size_kb.max {
        Some(m) => m.saturating_mul(1024),
        None => u64::MAX,
    };
    size >= min && size <= max
}

/// Matches a file's MIME type against a given MIME type string
pub(crate) fn match_mime_type(file_path: &Path, mime_type: &str) -> bool {
    log::debug!(
        "Matching file: {} against MIME type: {}",
        file_path.display(),
        mime_type
    );
    mime_guess::from_path(file_path)
        .first()
        .is_some_and(|mime| {
            let mime_essence = mime.essence_str();
            mime_type
                .strip_suffix("/*")
                .map_or(mime_essence == mime_type, |prefix| {
                    mime_essence.starts_with(prefix)
                })
        })
}

/// Helper function to parse date with fallback
fn parse_date_with_fallback(date_str: &str, fallback: NaiveDate) -> NaiveDate {
    parse_date(date_str).map_or_else(
        |_| {
            log::warn!("Invalid date format: {date_str}, using fallback");
            fallback
        },
        |dt| dt.date_naive(),
    )
}

/// Helper function to check if a date falls within a range
fn is_date_in_range(date: NaiveDate, date_range: &DateRange) -> bool {
    let from = date_range
        .from
        .as_ref()
        .map_or(*MIN_DATE_NAIVE, |from_str| {
            parse_date_with_fallback(from_str, *MIN_DATE_NAIVE)
        });

    let to = date_range.to.as_ref().map_or(*MAX_DATE_NAIVE, |to_str| {
        parse_date_with_fallback(to_str, *MAX_DATE_NAIVE)
    });

    date >= from && date <= to
}

/// Matches a file's metadata against a date range (created date)
pub(crate) fn match_date_range_created(metadata: &fs::Metadata, date_range: &DateRange) -> bool {
    log::debug!("Matching against created date range: {date_range:?}");

    metadata.created().is_ok_and(|created| {
        let created_datetime: chrono::DateTime<Utc> = created.into();
        let created_date = created_datetime.date_naive();
        is_date_in_range(created_date, date_range)
    })
}

/// Matches a file's metadata against a date range (modified date)
pub(crate) fn match_date_range_mod(metadata: &fs::Metadata, date_range: &DateRange) -> bool {
    log::debug!("Matching against modified date range: {date_range:?}");

    metadata.modified().is_ok_and(|modified| {
        let modified_datetime: chrono::DateTime<Utc> = modified.into();
        let modified_date = modified_datetime.date_naive();
        is_date_in_range(modified_date, date_range)
    })
}

/// Matches a file's symlink status against a boolean value
pub(crate) fn match_is_symlink(metadata: &fs::Metadata, is_symlink: bool) -> bool {
    log::debug!(
        "Matching symlink status: {} against expected: {}",
        metadata.file_type().is_symlink(),
        is_symlink
    );
    metadata.file_type().is_symlink() == is_symlink
}

/// Matches a specific metadata field (e.g., EXIF) against a file
pub(crate) fn match_metadata_field(file_path: &Path, field: &rule::MetadataField) -> bool {
    log::debug!(
        "Checking metadata field match for key '{}' on file '{}'",
        field.key,
        file_path.display()
    );

    let file = match fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            log::warn!("Failed to open file '{}': {}", file_path.display(), e);
            return false;
        }
    };

    let mut reader = BufReader::new(file);
    let exif = match Reader::new().read_from_container(&mut reader) {
        Ok(r) => r,
        Err(e) => {
            log::debug!("No EXIF data found in '{}': {}", file_path.display(), e);
            return false;
        }
    };

    let requested_key = field.key.to_lowercase();

    for f in exif.fields() {
        // `Display` gives the tag's name ("Make"); `Debug` would give
        // "Tag(Tiff, 271)", which no rule could ever match.
        let exif_key = format!("EXIF:{}", f.tag).to_lowercase();
        let value_str = f.display_value().with_unit(&exif).to_string();

        if exif_key == requested_key {
            log::debug!("Found EXIF key match: '{exif_key}'");

            if let Some(pattern_str) = &field.value {
                match Pattern::new(pattern_str) {
                    Ok(pattern) => {
                        // Text values are displayed in quotes ("Google"); match
                        // the plain text as well, so `value: Google` works.
                        let is_match = pattern.matches(&value_str)
                            || exif_text(&f.value).is_some_and(|text| pattern.matches(&text));
                        log::debug!(
                            "Comparing EXIF value '{value_str}' with pattern '{pattern_str}': {is_match}"
                        );
                        return is_match;
                    }
                    Err(e) => {
                        log::warn!("Invalid glob pattern '{pattern_str}': {e}");
                        return false;
                    }
                }
            }
            log::debug!("EXIF key '{exif_key}' matched without value filter");
            return true;
        }
    }

    log::debug!(
        "No matching EXIF key '{}' found in file '{}'",
        field.key,
        file_path.display()
    );

    false
}

/// Matches a file against all specified conditions in a rule.
///
/// Uses OR logic if `conditions.any` is true; otherwise AND logic.
pub fn match_rule_matcher(file_path: &Path, conditions: &Conditions) -> bool {
    log::debug!(
        "Matching file: {} against conditions: {:?}",
        file_path.display(),
        conditions
    );
    let metadata = match fs::symlink_metadata(file_path) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to read metadata for {}: {}", file_path.display(), e);
            return false;
        }
    };
    log::debug!("File metadata: {metadata:?}");

    // Only conditions that are actually set take part. Unset conditions must
    // not count as "matched", otherwise `any: true` would match every file.
    // A condition that fails to evaluate (e.g. an invalid regex) counts as
    // not matched.
    let ok = |r: Result<bool, TookaError>| r.unwrap_or(false);
    let mut results: Vec<bool> = Vec::new();
    if let Some(pattern) = &conditions.filename {
        results.push(ok(match_filename_regex(file_path, pattern)));
    }
    if let Some(exts) = &conditions.extensions {
        results.push(match_extensions(file_path, exts));
    }
    if let Some(pattern) = &conditions.path {
        results.push(ok(match_path(file_path, pattern)));
    }
    if let Some(size) = &conditions.size_kb {
        results.push(match_size_kb(&metadata, size));
    }
    if let Some(mime) = &conditions.mime_type {
        results.push(match_mime_type(file_path, mime));
    }
    if let Some(range) = &conditions.created_date {
        results.push(match_date_range_created(&metadata, range));
    }
    if let Some(range) = &conditions.modified_date {
        results.push(match_date_range_mod(&metadata, range));
    }
    if let Some(is_symlink) = conditions.is_symlink {
        results.push(match_is_symlink(&metadata, is_symlink));
    }
    if let Some(fields) = &conditions.metadata {
        results.push(
            fields
                .iter()
                .all(|field| match_metadata_field(file_path, field)),
        );
    }

    let any_conditions = conditions.any.unwrap_or(false);
    log::debug!("Conditions any: {any_conditions}, results: {results:?}");
    if results.is_empty() {
        // A rule without conditions matches every file.
        true
    } else if any_conditions {
        results.into_iter().any(|m| m)
    } else {
        results.into_iter().all(|m| m)
    }
}
