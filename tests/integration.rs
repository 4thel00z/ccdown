use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use ccdown::download::{download, download_paths, DownloadOptions};

/// Helper: create a gzip file containing lines (simulating a .paths.gz file)
fn create_paths_gz(dir: &Path, filename: &str, lines: &[&str]) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    let file = std::fs::File::create(&file_path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    for line in lines {
        writeln!(encoder, "{}", line).unwrap();
    }
    encoder.finish().unwrap();
    file_path
}

#[tokio::test]
async fn download_paths_fetches_gz_file() {
    let server = MockServer::start().await;
    let body = b"fake gzip content for paths";

    // Mock HEAD (for status check)
    Mock::given(method("HEAD"))
        .and(path("/crawl-data/CC-MAIN-2025-08/warc.paths.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    // Mock GET (for actual download)
    Mock::given(method("GET"))
        .and(path("/crawl-data/CC-MAIN-2025-08/warc.paths.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let dst = TempDir::new().unwrap();

    let options = DownloadOptions {
        snapshot: "CC-MAIN-2025-08".to_string(),
        data_type: "warc",
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        ..Default::default()
    };

    download_paths(options).await.unwrap();

    let downloaded = dst.path().join("warc.paths.gz");
    assert!(downloaded.exists(), "paths file should be downloaded");
    let content = std::fs::read(&downloaded).unwrap();
    assert_eq!(content, body);
}

#[tokio::test]
async fn download_paths_reformats_news_snapshot() {
    let server = MockServer::start().await;
    let body = b"news paths content";

    // CC-NEWS-2025-01 should become CC-NEWS/2025/01 in the URL
    Mock::given(method("HEAD"))
        .and(path("/crawl-data/CC-NEWS/2025/01/warc.paths.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/crawl-data/CC-NEWS/2025/01/warc.paths.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let dst = TempDir::new().unwrap();

    let options = DownloadOptions {
        snapshot: "CC-NEWS-2025-01".to_string(),
        data_type: "warc",
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        ..Default::default()
    };

    download_paths(options).await.unwrap();

    let downloaded = dst.path().join("warc.paths.gz");
    assert!(downloaded.exists());
    assert_eq!(std::fs::read(&downloaded).unwrap(), body);
}

#[tokio::test]
async fn download_paths_returns_error_on_404() {
    let server = MockServer::start().await;

    Mock::given(method("HEAD"))
        .and(path("/crawl-data/CC-MAIN-2099-01/warc.paths.gz"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let dst = TempDir::new().unwrap();

    let options = DownloadOptions {
        snapshot: "CC-MAIN-2099-01".to_string(),
        data_type: "warc",
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        ..Default::default()
    };

    let result = download_paths(options).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Doesn't seem to exist"),
        "expected 404 message, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn download_fetches_files_from_paths_gz() {
    let server = MockServer::start().await;

    let file_a_body = b"contents of file A";
    let file_b_body = b"contents of file B";

    // Mock HEAD + GET for two files
    for (file_path, body) in [
        ("data/fileA.warc.gz", file_a_body.as_slice()),
        ("data/fileB.warc.gz", file_b_body.as_slice()),
    ] {
        Mock::given(method("HEAD"))
            .and(path(format!("/{}", file_path)))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-length", body.len().to_string().as_str())
                    .set_body_bytes(body.to_vec()),
            )
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/{}", file_path)))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
            .mount(&server)
            .await;
    }

    let tmp = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();

    // Create a .paths.gz file with relative paths (no leading slash, no base URL)
    let paths_file = create_paths_gz(
        tmp.path(),
        "warc.paths.gz",
        &["data/fileA.warc.gz", "data/fileB.warc.gz"],
    );

    let options = DownloadOptions {
        paths: &paths_file,
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        threads: 2,
        ..Default::default()
    };

    download(options).await.unwrap();

    // Files should be at dst/data/fileA.warc.gz and dst/data/fileB.warc.gz
    let file_a = dst.path().join("data/fileA.warc.gz");
    let file_b = dst.path().join("data/fileB.warc.gz");
    assert!(file_a.exists(), "fileA should be downloaded");
    assert!(file_b.exists(), "fileB should be downloaded");
    assert_eq!(std::fs::read(&file_a).unwrap(), file_a_body);
    assert_eq!(std::fs::read(&file_b).unwrap(), file_b_body);
}

#[tokio::test]
async fn download_files_only_flattens_structure() {
    let server = MockServer::start().await;
    let body = b"flat file content";

    Mock::given(method("HEAD"))
        .and(path("/crawl-data/segment/1234/file.warc.gz"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", body.len().to_string().as_str())
                .set_body_bytes(body.to_vec()),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/crawl-data/segment/1234/file.warc.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();

    let paths_file = create_paths_gz(
        tmp.path(),
        "warc.paths.gz",
        &["crawl-data/segment/1234/file.warc.gz"],
    );

    let options = DownloadOptions {
        paths: &paths_file,
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        files_only: true,
        ..Default::default()
    };

    download(options).await.unwrap();

    // files_only should put it directly in dst, not nested
    let flat_file = dst.path().join("file.warc.gz");
    assert!(
        flat_file.exists(),
        "file should be directly in dst with files_only"
    );
    assert_eq!(std::fs::read(&flat_file).unwrap(), body);
}

#[tokio::test]
async fn download_numbered_renames_files() {
    let server = MockServer::start().await;
    let body = b"numbered content";

    Mock::given(method("HEAD"))
        .and(path("/data/some/deep/path.wet.gz"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", body.len().to_string().as_str())
                .set_body_bytes(body.to_vec()),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/data/some/deep/path.wet.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();

    let paths_file = create_paths_gz(
        tmp.path(),
        "wet.paths.gz",
        &["data/some/deep/path.wet.gz"],
    );

    let options = DownloadOptions {
        paths: &paths_file,
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        numbered: true,
        ..Default::default()
    };

    download(options).await.unwrap();

    // numbered mode: first file should be 0.txt.gz
    let numbered_file = dst.path().join("0.txt.gz");
    assert!(
        numbered_file.exists(),
        "file should be named 0.txt.gz in numbered mode"
    );
    assert_eq!(std::fs::read(&numbered_file).unwrap(), body);
}

#[tokio::test]
async fn download_strict_mode_aborts_on_404() {
    let server = MockServer::start().await;

    let good_body = b"good file";

    // First file succeeds
    Mock::given(method("HEAD"))
        .and(path("/data/good.warc.gz"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", good_body.len().to_string().as_str())
                .set_body_bytes(good_body.to_vec()),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/data/good.warc.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(good_body.to_vec()))
        .mount(&server)
        .await;

    // Second file 404s
    Mock::given(method("HEAD"))
        .and(path("/data/missing.warc.gz"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();

    let paths_file = create_paths_gz(
        tmp.path(),
        "warc.paths.gz",
        &["data/good.warc.gz", "data/missing.warc.gz"],
    );

    let options = DownloadOptions {
        paths: &paths_file,
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        threads: 1, // serialize to make the test deterministic
        strict: true,
        ..Default::default()
    };

    let result = download(options).await;
    assert!(result.is_err(), "strict mode should return error on 404");

    let err = result.unwrap_err();
    assert!(
        err.is_unrecoverable(),
        "error should be marked unrecoverable"
    );
}

#[tokio::test]
async fn download_non_strict_continues_on_404() {
    let server = MockServer::start().await;

    let good_body = b"good file content";

    // good file
    Mock::given(method("HEAD"))
        .and(path("/data/good.warc.gz"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", good_body.len().to_string().as_str())
                .set_body_bytes(good_body.to_vec()),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/data/good.warc.gz"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(good_body.to_vec()))
        .mount(&server)
        .await;

    // missing file
    Mock::given(method("HEAD"))
        .and(path("/data/missing.warc.gz"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();

    let paths_file = create_paths_gz(
        tmp.path(),
        "warc.paths.gz",
        &["data/good.warc.gz", "data/missing.warc.gz"],
    );

    let options = DownloadOptions {
        paths: &paths_file,
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        threads: 2,
        strict: false,
        ..Default::default()
    };

    let result = download(options).await;
    assert!(
        result.is_ok(),
        "non-strict mode should succeed despite 404: {:?}",
        result
    );

    // The good file should still have been downloaded
    let good_file = dst.path().join("data/good.warc.gz");
    assert!(good_file.exists(), "good file should still be downloaded");
    assert_eq!(std::fs::read(&good_file).unwrap(), good_body);
}

#[tokio::test]
async fn download_strict_mode_aborts_on_403() {
    let server = MockServer::start().await;

    Mock::given(method("HEAD"))
        .and(path("/data/forbidden.warc.gz"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();

    let paths_file = create_paths_gz(
        tmp.path(),
        "warc.paths.gz",
        &["data/forbidden.warc.gz"],
    );

    let options = DownloadOptions {
        paths: &paths_file,
        dst: dst.path(),
        base_url: Some(format!("{}/", server.uri())),
        max_retries: 1,
        strict: true,
        ..Default::default()
    };

    let result = download(options).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_unrecoverable());
    assert!(err.to_string().contains("403"));
}
