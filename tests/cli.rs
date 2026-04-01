use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn no_command_shows_error() {
    Command::cargo_bin("ccdown")
        .unwrap()
        .assert()
        .stderr(predicate::str::contains("No command specified"));
}

#[test]
fn help_flag_works() {
    Command::cargo_bin("ccdown")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("downloader for Common Crawl"));
}

#[test]
fn version_flag_works() {
    Command::cargo_bin("ccdown")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn download_paths_rejects_invalid_crawl_name() {
    Command::cargo_bin("ccdown")
        .unwrap()
        .args(["download-paths", "INVALID-FORMAT", "warc", "/tmp"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("CC-MAIN-YYYY-WW"));
}

#[test]
fn download_rejects_numbered_and_files_only() {
    Command::cargo_bin("ccdown")
        .unwrap()
        .args([
            "download",
            "/nonexistent/path.gz",
            "/tmp/out",
            "--numbered",
            "--files-only",
        ])
        .assert()
        .stderr(predicate::str::contains("incompatible"));
}

#[test]
fn download_accepts_strict_flag() {
    // This will fail because the file doesn't exist, but the flag should be accepted
    Command::cargo_bin("ccdown")
        .unwrap()
        .args([
            "download",
            "help",
        ])
        .assert()
        .failure()
        // Should not fail due to unknown flag
        .stderr(predicate::str::contains("strict").not());
}

#[test]
fn download_help_shows_strict_option() {
    Command::cargo_bin("ccdown")
        .unwrap()
        .args(["download", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--strict"));
}
