use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("crypto-extractor"));
    assert!(stdout.contains("Cryptographic parameter extractor"));
    assert!(stdout.contains("--path"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--output-file"));
    assert!(stdout.contains("--language"));
}

#[test]
fn test_cli_missing_path() {
    let output = Command::new("cargo")
        .args(["run", "--"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("--path"));
}

#[test]
fn test_cli_invalid_path() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--path",
            "/nonexistent/path/that/does/not/exist",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("does not exist") || stderr.contains("Invalid arguments"));
}

#[test]
fn test_cli_invalid_language() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.go");
    fs::write(&file_path, "package main").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--path",
            file_path.to_str().unwrap(),
            "--language",
            "java",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Unsupported language")
            || stderr.contains("Invalid arguments")
            || stderr.contains("invalid value")
            || stderr.contains("possible values")
    );
}

#[test]
fn test_cli_invalid_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.go");
    fs::write(&file_path, "package main").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--path",
            file_path.to_str().unwrap(),
            "--format",
            "xml",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Unsupported output format")
            || stderr.contains("Invalid arguments")
            || stderr.contains("invalid value")
            || stderr.contains("possible values")
    );
}

#[test]
fn test_cli_valid_args() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.go");
    fs::write(&file_path, "package main\n\nfunc main() {}").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--path",
            file_path.to_str().unwrap(),
            "--language",
            "go",
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to execute command");

    // Should parse successfully (may fail later in processing, but CLI args should be valid)
    let stderr = String::from_utf8(output.stderr).unwrap();
    // If it fails, it should be after validation (e.g., parsing errors, not argument errors)
    assert!(!stderr.contains("Invalid arguments") && !stderr.contains("Unsupported"));
}
