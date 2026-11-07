//! Tests for file collection and report generation.
//! Sorting itself is tested in `core::engine::tests`.
#[cfg(test)]
mod tests {
    use crate::core::error::TookaError;
    use crate::core::sorter::{MatchResult, collect_files};
    use crate::utils::gen_pdf::generate_pdf;
    use std::fs::{File, create_dir_all};
    use std::io::Write;
    use tempfile::tempdir;

    /// Helper function to create a test file with content
    fn create_test_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_collect_files() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path();

        // Create test files in subdirectories
        let sub_dir = source_path.join("subdir");
        create_dir_all(&sub_dir).unwrap();

        let files = [
            source_path.join("file1.txt"),
            source_path.join("file2.log"),
            sub_dir.join("file3.data"),
        ];

        for file in &files {
            create_test_file(file, "content").unwrap();
        }

        // Collect files
        let collected = collect_files(source_path).expect("collect_files should succeed");

        // Should find all files
        assert_eq!(collected.len(), 3);

        // Check that all created files are found
        for file in &files {
            assert!(
                collected.contains(file),
                "Should find file: {}",
                file.display()
            );
        }
    }

    #[test]
    fn test_collect_files_nonexistent_directory() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        // Should return an error for nonexistent directory
        let result = collect_files(&nonexistent_path);
        assert!(result.is_err());

        if let Err(TookaError::ConfigError(msg)) = result {
            assert!(msg.contains("does not exist"));
        } else {
            panic!("Expected ConfigError");
        }
    }

    #[test]
    fn test_pdf_generation() {
        let temp_dir = tempdir().unwrap();
        let row = |name: &str, action: &str, rule: &str, to: &str| MatchResult {
            file_name: name.into(),
            action: action.into(),
            matched_rule_id: rule.into(),
            current_path: temp_dir.path().join(name),
            new_path: temp_dir.path().join(to).join(name),
        };
        let results = vec![
            row("test1.txt", "move", "txt_rule", "txt_files"),
            row("test2.log", "copy", "log_rule", "log_files"),
            row("test3.data", "move", "data_rule", "data_files"),
            row("test5.unknown", "skip", "none", ""),
        ];

        let pdf_path = temp_dir.path().join("test_report.pdf");
        generate_pdf(&pdf_path, &results).expect("PDF generation should succeed");

        let metadata = std::fs::metadata(&pdf_path).expect("Should be able to read PDF metadata");
        assert!(
            metadata.len() > 500,
            "PDF should have reasonable size (> 500 bytes), actual size: {} bytes",
            metadata.len()
        );
        let bytes = std::fs::read(&pdf_path).unwrap();
        assert!(bytes.starts_with(b"%PDF-"), "output should be a PDF file");
    }

    #[test]
    fn test_pdf_generation_with_large_dataset() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create multiple results to test pagination and grouping
        let mut mock_results = Vec::new();

        // Create results for multiple rules with different actions
        for i in 0..50 {
            mock_results.push(MatchResult {
                file_name: format!("file{i}.txt"),
                current_path: source_path.join(format!("file{i}.txt")),
                new_path: source_path.join("txt_files").join(format!("file{i}.txt")),
                matched_rule_id: "txt_rule".to_string(),
                action: "move".to_string(),
            });
        }

        for i in 0..30 {
            mock_results.push(MatchResult {
                file_name: format!("log{i}.log"),
                current_path: source_path.join(format!("log{i}.log")),
                new_path: source_path.join("log_files").join(format!("log{i}.log")),
                matched_rule_id: "log_rule".to_string(),
                action: "copy".to_string(),
            });
        }

        for i in 0..20 {
            mock_results.push(MatchResult {
                file_name: format!("data{i}.data"),
                current_path: source_path.join(format!("data{i}.data")),
                new_path: source_path.join("data_files").join(format!("data{i}.data")),
                matched_rule_id: "data_rule".to_string(),
                action: "move".to_string(),
            });
        }

        for i in 0..10 {
            mock_results.push(MatchResult {
                file_name: format!("executable{i}.exe"),
                current_path: source_path.join(format!("executable{i}.exe")),
                new_path: source_path
                    .join("executed")
                    .join(format!("executed_{i}.exe")),
                matched_rule_id: "execute_rule".to_string(),
                action: "execute".to_string(),
            });
        }

        for i in 0..15 {
            mock_results.push(MatchResult {
                file_name: format!("unknown{i}.unknown"),
                current_path: source_path.join(format!("unknown{i}.unknown")),
                new_path: source_path.join(format!("unknown{i}.unknown")), // Same path for skip
                matched_rule_id: "none".to_string(),
                action: "skip".to_string(),
            });
        }

        // Generate PDF with large dataset
        let pdf_path = temp_dir.path().join("large_test_report.pdf");
        generate_pdf(&pdf_path, &mock_results)
            .expect("PDF generation should succeed with large dataset");

        // Verify PDF file was created
        assert!(pdf_path.exists(), "Large PDF file should be created");

        // Check that the PDF file has reasonable size
        let metadata =
            std::fs::metadata(&pdf_path).expect("Should be able to read large PDF metadata");
        assert!(
            metadata.len() > 5000,
            "Large PDF should have substantial size (> 5KB), actual size: {} bytes",
            metadata.len()
        );

        println!("Generated large PDF at: {}", pdf_path.display());
        println!("Large PDF size: {} bytes", metadata.len());
        println!("Total match results: {}", mock_results.len());

        // Verify we have the expected number of results
        assert_eq!(mock_results.len(), 125); // 50+30+20+10+15

        // Verify different action types are present
        let move_count = mock_results.iter().filter(|r| r.action == "move").count();
        let copy_count = mock_results.iter().filter(|r| r.action == "copy").count();
        let execute_count = mock_results
            .iter()
            .filter(|r| r.action == "execute")
            .count();
        let skip_count = mock_results.iter().filter(|r| r.action == "skip").count();

        assert_eq!(move_count, 70); // txt + data files
        assert_eq!(copy_count, 30); // log files
        assert_eq!(execute_count, 10); // executable files
        assert_eq!(skip_count, 15); // unknown files
    }

    /// Where to write a PDF meant for manual inspection.
    ///
    /// By default the PDF goes into a temporary directory that is cleaned up
    /// when the returned guard is dropped. Set `TOOKA_TEST_PDF_DIR` to keep the
    /// generated PDFs in that directory instead, e.g.
    /// `TOOKA_TEST_PDF_DIR=. cargo test pdf_generation`.
    fn inspection_pdf_path(name: &str) -> (Option<tempfile::TempDir>, std::path::PathBuf) {
        match std::env::var_os("TOOKA_TEST_PDF_DIR") {
            Some(dir) => {
                let dir = std::path::PathBuf::from(dir);
                std::fs::create_dir_all(&dir).expect("Should create TOOKA_TEST_PDF_DIR");
                (None, dir.join(name))
            }
            None => {
                let tmp = tempfile::tempdir().expect("Should create temp dir");
                let path = tmp.path().join(name);
                (Some(tmp), path)
            }
        }
    }

    #[test]
    fn test_pdf_generation_for_inspection() {
        // Set TOOKA_TEST_PDF_DIR to keep the PDF for manual inspection
        let (_tmp, pdf_path) = inspection_pdf_path("test_report_refactored.pdf");
        let pdf_path = pdf_path.as_path();

        // Create mock results with various actions to test all features
        let mut mock_results = Vec::new();
        let base_path = std::path::Path::new("/example/source");

        // Create results for different rule types and actions
        for i in 0..15 {
            mock_results.push(MatchResult {
                file_name: format!("document_{i}.txt"),
                current_path: base_path.join(format!("documents/document_{i}.txt")),
                new_path: base_path
                    .join("organized/documents")
                    .join(format!("document_{i}.txt")),
                matched_rule_id: "document_organization_rule".to_string(),
                action: "move".to_string(),
            });
        }

        for i in 0..12 {
            mock_results.push(MatchResult {
                file_name: format!("backup_{i}.log"),
                current_path: base_path.join(format!("logs/backup_{i}.log")),
                new_path: base_path
                    .join("archive/logs")
                    .join(format!("backup_{i}.log")),
                matched_rule_id: "log_backup_rule".to_string(),
                action: "copy".to_string(),
            });
        }

        for i in 0..8 {
            mock_results.push(MatchResult {
                file_name: format!("temp_{i}.tmp"),
                current_path: base_path.join(format!("temp/temp_{i}.tmp")),
                new_path: base_path.join("temp").join(format!("temp_{i}.tmp")), // Same path for delete
                matched_rule_id: "cleanup_rule".to_string(),
                action: "delete".to_string(),
            });
        }

        for i in 0..6 {
            mock_results.push(MatchResult {
                file_name: format!("old_file_{i}.dat"),
                current_path: base_path.join(format!("data/old_file_{i}.dat")),
                new_path: base_path.join("data").join(format!("new_file_{i}.dat")),
                matched_rule_id: "rename_rule".to_string(),
                action: "rename".to_string(),
            });
        }

        for i in 0..5 {
            mock_results.push(MatchResult {
                file_name: format!("script_{i}.sh"),
                current_path: base_path.join(format!("scripts/script_{i}.sh")),
                new_path: base_path
                    .join("executed")
                    .join(format!("executed_script_{i}.result")),
                matched_rule_id: "script_execution_rule".to_string(),
                action: "execute".to_string(),
            });
        }

        for i in 0..10 {
            mock_results.push(MatchResult {
                file_name: format!("unknown_{i}.xyz"),
                current_path: base_path.join(format!("misc/unknown_{i}.xyz")),
                new_path: base_path.join("misc").join(format!("unknown_{i}.xyz")), // Same path for skip
                matched_rule_id: "none".to_string(),
                action: "skip".to_string(),
            });
        }

        // Generate PDF
        generate_pdf(pdf_path, &mock_results).expect("PDF generation should succeed");

        // Verify PDF file was created
        assert!(pdf_path.exists(), "PDF file should be created");

        // Check file size
        let metadata = std::fs::metadata(pdf_path).expect("Should be able to read PDF metadata");

        println!("Generated PDF for inspection at: {}", pdf_path.display());
        println!("PDF size: {} bytes", metadata.len());
        println!("Total match results: {}", mock_results.len());

        // Show action breakdown
        let move_count = mock_results.iter().filter(|r| r.action == "move").count();
        let copy_count = mock_results.iter().filter(|r| r.action == "copy").count();
        let delete_count = mock_results.iter().filter(|r| r.action == "delete").count();
        let rename_count = mock_results.iter().filter(|r| r.action == "rename").count();
        let execute_count = mock_results
            .iter()
            .filter(|r| r.action == "execute")
            .count();
        let skip_count = mock_results.iter().filter(|r| r.action == "skip").count();

        println!("Action breakdown:");
        println!("  Move: {move_count}");
        println!("  Copy: {copy_count}");
        println!("  Delete: {delete_count}");
        println!("  Rename: {rename_count}");
        println!("  Execute: {execute_count}");
        println!("  Skip: {skip_count}");

        assert!(
            metadata.len() > 2000,
            "PDF should be substantial for inspection"
        );
    }

    #[test]
    fn test_pdf_generation_with_long_paths() {
        // Create PDF with extremely long paths to test wrapping
        let (_tmp, pdf_path) = inspection_pdf_path("test_report_long_paths.pdf");
        let pdf_path = pdf_path.as_path();

        let mock_results = vec![
            MatchResult {
                file_name:
                    "document_with_very_very_very_long_filename_that_should_be_handled_properly.txt"
                        .to_string(),
                current_path: std::path::PathBuf::from(
                    "/home/user/documents/projects/rust/my_awesome_project/src/very/deeply/nested/directories/with/extremely/long/path/names/that/go/on/and/on/document_with_very_very_very_long_filename_that_should_be_handled_properly.txt",
                ),
                new_path: std::path::PathBuf::from(
                    "/home/user/organized_files/documents/text_files/2024/august/important_documents/document_with_very_very_very_long_filename_that_should_be_handled_properly.txt",
                ),
                matched_rule_id: "document_organization_with_very_long_rule_name".to_string(),
                action: "move".to_string(),
            },
            MatchResult {
                file_name: "short.log".to_string(),
                current_path: std::path::PathBuf::from(
                    "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z/very/long/path/structure/short.log",
                ),
                new_path: std::path::PathBuf::from("/backup/logs/short.log"),
                matched_rule_id: "log_backup".to_string(),
                action: "copy".to_string(),
            },
            MatchResult {
                file_name: "file_in_normal_path.dat".to_string(),
                current_path: std::path::PathBuf::from("/home/user/data/file_in_normal_path.dat"),
                new_path: std::path::PathBuf::from("/home/user/archived/file_in_normal_path.dat"),
                matched_rule_id: "normal_rule".to_string(),
                action: "move".to_string(),
            },
        ];

        // Generate PDF
        generate_pdf(pdf_path, &mock_results).expect("PDF generation should succeed");

        // Verify PDF file was created
        assert!(pdf_path.exists(), "PDF file should be created");

        // Check file size
        let metadata = std::fs::metadata(pdf_path).expect("Should be able to read PDF metadata");

        println!("Generated PDF with long paths at: {}", pdf_path.display());
        println!("PDF size: {} bytes", metadata.len());
        println!("Total match results: {}", mock_results.len());

        assert!(
            metadata.len() > 1000,
            "PDF should be substantial for inspection"
        );
    }
}
