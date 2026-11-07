use crate::core::error::TookaError;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use exif::{In, Reader, Tag};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// Cached regex pattern for template matching
static TEMPLATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{(.*?)\}\}").expect("Failed to compile template regex"));

/// Evaluates a template string with metadata and file information.
pub(crate) fn evaluate_template(
    template: &str,
    file_path: &Path,
    metadata: &HashMap<String, String>,
) -> String {
    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let mut result = template.to_string();

    for caps in TEMPLATE_REGEX.captures_iter(template) {
        let full_match = &caps[0];
        let expr = &caps[1];

        let mut parts = expr.split('|');
        let key = parts.next().unwrap().trim();
        let filters: Vec<&str> = parts.collect();

        let raw_value = if key == "filename" {
            file_name.clone()
        } else if let Some(metadata_key) = key.strip_prefix("metadata.") {
            metadata.get(metadata_key).cloned().unwrap_or_default()
        } else {
            String::new()
        };

        let final_value = apply_filters(raw_value, &filters);
        result = result.replace(full_match, &final_value);
    }

    result
}

fn apply_filters(value: String, filters: &[&str]) -> String {
    let mut val = value;
    for filter in filters {
        if let Some(fmt) = filter.strip_prefix("date:") {
            // Try RFC3339 first
            let datetime = DateTime::parse_from_rfc3339(&val)
                .ok()
                .map(|dt| dt.with_timezone(&Local))
                .or_else(|| {
                    // EXIF dates as shown by kamadak-exif ("2024-05-01 12:30:00")
                    // and in their raw EXIF form ("2024:05:01 12:30:00").
                    ["%Y-%m-%d %H:%M:%S", "%Y:%m:%d %H:%M:%S"]
                        .iter()
                        .find_map(|f| NaiveDateTime::parse_from_str(&val, f).ok())
                        .and_then(|dt| Local.from_local_datetime(&dt).single())
                });

            if let Some(dt) = datetime {
                val = dt.format(fmt).to_string();
            }
        }
    }
    val
}

/// Returns metadata fields for use in templating
pub(crate) fn extract_metadata(file_path: &Path) -> Result<HashMap<String, String>, TookaError> {
    let mut map = HashMap::new();

    // File system metadata
    let metadata = fs::metadata(file_path)?;

    if let Ok(modified) = metadata.modified() {
        let dt: DateTime<Local> = modified.into();
        map.insert("modified".into(), dt.to_rfc3339());
    }

    if let Ok(created) = metadata.created() {
        let dt: DateTime<Local> = created.into();
        map.insert("created".into(), dt.to_rfc3339());
    }

    map.insert("size".into(), metadata.len().to_string());

    // Attempt to parse EXIF (for JPEG/PNG)
    if let Ok(file) = fs::File::open(file_path)
        && let Ok(reader) = Reader::new().read_from_container(&mut std::io::BufReader::new(file))
    {
        for field in reader.fields() {
            // e.g. "EXIF:Make" -> "Google" (text without the display quotes)
            let key = format!("EXIF:{}", field.tag);
            let value = crate::file::file_match::exif_text(&field.value)
                .unwrap_or_else(|| field.display_value().with_unit(&reader).to_string());
            map.insert(key, value);
        }

        // Common aliases for convenience
        if let Some(field) = reader.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
            map.insert("EXIF:DateTime".into(), field.display_value().to_string());
        }
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(template: &str, key: &str, value: &str) -> String {
        let metadata = HashMap::from([(key.to_string(), value.to_string())]);
        evaluate_template(template, Path::new("/x/photo.jpg"), &metadata)
    }

    #[test]
    fn filename_is_the_name_without_extension() {
        assert_eq!(render("{{filename}}-1.jpg", "k", "v"), "photo-1.jpg");
    }

    #[test]
    fn date_filter_understands_exif_dates() {
        for raw in ["2024-05-01 12:30:00", "2024:05:01 12:30:00"] {
            assert_eq!(
                render(
                    "{{metadata.EXIF:DateTime|date:%Y-%m-%d}}_{{filename}}",
                    "EXIF:DateTime",
                    raw
                ),
                "2024-05-01_photo"
            );
        }
    }

    #[test]
    fn date_filter_understands_file_times() {
        assert_eq!(
            render(
                "{{metadata.modified|date:%Y}}",
                "modified",
                "2023-02-03T04:05:06+00:00"
            ),
            "2023"
        );
    }

    #[test]
    fn unknown_placeholders_become_empty() {
        assert_eq!(render("a{{nope}}b{{metadata.missing}}c", "k", "v"), "abc");
    }
}
