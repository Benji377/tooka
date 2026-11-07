//! YAML serialization helpers.
//!
//! Wraps `serde_saphyr` so every YAML file Tooka writes uses the same style.
//! Long strings (such as paths) are kept as plain one-line scalars instead of
//! folded `>-` blocks, which keeps config and rule files easy to edit by hand
//! and close to the format written by earlier Tooka versions.

use serde::Serialize;
use serde_saphyr::{SerializeError, SerializerOptions};

fn options() -> SerializerOptions {
    let mut options = SerializerOptions::default();
    options.prefer_block_scalars = false;
    options
}

/// Serializes `value` to a YAML string.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, SerializeError> {
    serde_saphyr::to_string_with_options(value, options())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Sample {
        path: String,
        items: Vec<String>,
    }

    #[test]
    fn long_strings_stay_plain_and_round_trip() {
        let sample = Sample {
            path: format!("/home/user/{}/some folder/file.txt", "a".repeat(120)),
            items: vec!["pdf".into()],
        };
        let yaml = to_string(&sample).unwrap();
        assert!(!yaml.contains(">-"), "unexpected folded scalar:\n{yaml}");
        assert_eq!(
            yaml.lines().count(),
            3,
            "expected one line per value:\n{yaml}"
        );
        let back: Sample = serde_saphyr::from_str(&yaml).unwrap();
        assert_eq!(back, sample);
    }
}
