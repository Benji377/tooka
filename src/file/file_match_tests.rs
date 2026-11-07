use std::fs::{self};
use std::io::Write;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir, tempdir};

use super::file_match;
use crate::rules::rule::{Conditions, DateRange, MetadataField, Range};

/// Creates `rel` (e.g. "photos/a.jpg") inside `dir`, with parent folders.
fn create_file(dir: &TempDir, rel: &str) -> PathBuf {
    let path = dir.path().join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "").unwrap();
    path
}

#[test]
fn test_match_filename_regex() {
    let dir = tempdir().unwrap();
    let matching_path = create_file(&dir, "match_test.jpg");
    let non_matching_path = create_file(&dir, "fail_test.png");

    assert!(file_match::match_filename_regex(&matching_path, r"match_.*\.jpg").unwrap());
    assert!(!file_match::match_filename_regex(&non_matching_path, r"match_.*\.jpg").unwrap());
}

#[test]
fn test_match_extensions() {
    let dir = tempdir().unwrap();
    let matching_path = create_file(&dir, "a.jpg");
    let non_matching_path = create_file(&dir, "a.png");

    assert!(file_match::match_extensions(
        &matching_path,
        &["jpg".to_string()]
    ));
    assert!(!file_match::match_extensions(
        &non_matching_path,
        &["jpg".to_string()]
    ));
}

#[test]
fn test_match_path() {
    let dir = tempdir().unwrap();
    let matching_path = create_file(&dir, "photos/match.jpg");
    let non_matching_path = create_file(&dir, "docs/fail.txt");

    assert!(file_match::match_path(&matching_path, "**/photos/*.jpg").unwrap());
    assert!(!file_match::match_path(&non_matching_path, "**/photos/*.jpg").unwrap());
}

#[test]
fn test_match_size_kb() {
    let mut small_file = NamedTempFile::new().unwrap();
    let mut large_file = NamedTempFile::new().unwrap();

    writeln!(small_file, "tiny").unwrap();
    writeln!(large_file, "{}", "x".repeat(1024 * 10)).unwrap(); // 10 KB

    let small_meta = small_file.as_file().metadata().unwrap();
    let large_meta = large_file.as_file().metadata().unwrap();

    let range = Range {
        min: Some(5), // 5 KB
        max: Some(20),
    };

    assert!(!file_match::match_size_kb(&small_meta, &range));
    assert!(file_match::match_size_kb(&large_meta, &range));
}

#[test]
fn test_match_mime_type() {
    let dir = tempdir().unwrap();
    let jpg_path = create_file(&dir, "a.jpg");
    let txt_path = create_file(&dir, "a.txt");

    assert!(file_match::match_mime_type(&jpg_path, "image/*"));
    assert!(!file_match::match_mime_type(&txt_path, "image/*"));
}

#[test]
fn test_match_date_range_mod() {
    let file = NamedTempFile::new().unwrap();
    let meta = file.as_file().metadata().unwrap();

    let today = chrono::Utc::now().naive_utc().date();

    let range = DateRange {
        from: Some(today.format("%Y-%m-%d").to_string()),
        to: Some(today.format("%Y-%m-%d").to_string()),
    };

    assert!(file_match::match_date_range_mod(&meta, &range));
}

#[test]
fn test_match_date_range_created() {
    let file = NamedTempFile::new().unwrap();
    let meta = file.as_file().metadata().unwrap();

    let today = chrono::Utc::now().naive_utc().date();

    let range = DateRange {
        from: Some(today.format("%Y-%m-%d").to_string()),
        to: Some(today.format("%Y-%m-%d").to_string()),
    };

    // Not every filesystem records a creation time; without one the
    // condition never matches. With one, a file created now is in range.
    let result = file_match::match_date_range_created(&meta, &range);
    assert_eq!(result, meta.created().is_ok());
}

#[test]
fn test_match_is_symlink() {
    let dir = tempdir().unwrap();
    let file = create_file(&dir, "target.txt");
    let symlink_path = dir.path().join("link.symlink");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&file, &symlink_path).unwrap();

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&file, &symlink_path).unwrap();

    let file_meta = fs::symlink_metadata(&file).unwrap();
    let symlink_meta = fs::symlink_metadata(&symlink_path).unwrap();

    assert!(!file_match::match_is_symlink(&file_meta, true));
    assert!(file_match::match_is_symlink(&symlink_meta, true));
}

#[test]
fn test_match_metadata_field_nonexistent() {
    let dir = tempdir().unwrap();
    let path = create_file(&dir, "blank.jpg");

    let field = MetadataField {
        key: "EXIF:DateTimeOriginal".to_string(),
        value: Some("*".to_string()),
    };

    // No EXIF data in a blank temp file
    assert!(!file_match::match_metadata_field(&path, &field));
}

fn conditions() -> Conditions {
    Conditions {
        any: None,
        filename: None,
        extensions: None,
        path: None,
        size_kb: None,
        mime_type: None,
        created_date: None,
        modified_date: None,
        is_symlink: None,
        metadata: None,
    }
}

/// Regression: with `any: true`, conditions that are not set used to count
/// as matched, so every file matched.
#[test]
fn any_mode_only_considers_conditions_that_are_set() {
    let dir = tempdir().unwrap();
    let pdf = create_file(&dir, "a.pdf");
    let txt = create_file(&dir, "a.txt");

    let mut or_conditions = conditions();
    or_conditions.any = Some(true);
    or_conditions.extensions = Some(vec!["pdf".into()]);
    or_conditions.filename = Some("^report".into());
    assert!(file_match::match_rule_matcher(&pdf, &or_conditions));
    assert!(!file_match::match_rule_matcher(&txt, &or_conditions));

    let mut and_conditions = or_conditions.clone();
    and_conditions.any = Some(false);
    assert!(!file_match::match_rule_matcher(&pdf, &and_conditions));

    assert!(
        file_match::match_rule_matcher(&txt, &conditions()),
        "a rule without conditions matches every file"
    );
}

#[test]
fn exif_text_values_are_compared_without_quotes() {
    let value = exif::Value::Ascii(vec![b"Google".to_vec()]);
    assert_eq!(file_match::exif_text(&value).as_deref(), Some("Google"));
    assert_eq!(file_match::exif_text(&exif::Value::Short(vec![1])), None);
}
