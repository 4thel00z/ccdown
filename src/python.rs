use pyo3::prelude::*;
use std::path::{Path, PathBuf};

use crate::download;
use crate::errors::DownloadError;

fn run_async<F: std::future::Future<Output = Result<(), DownloadError>>>(
    fut: F,
) -> PyResult<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    rt.block_on(fut)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Python client for downloading Common Crawl data.
///
/// Example:
///     client = Client(threads=10, retries=1000, progress=True)
///     client.paths("CC-MAIN-2025-08", "warc").to("./paths")
///     client.download("./paths/warc.paths.gz").to("./data")
#[pyclass]
#[derive(Clone)]
pub struct Client {
    threads: usize,
    retries: usize,
    progress: bool,
}

#[pymethods]
impl Client {
    #[new]
    #[pyo3(signature = (threads=10, retries=1000, progress=false))]
    fn new(threads: usize, retries: usize, progress: bool) -> Self {
        Client {
            threads,
            retries,
            progress,
        }
    }

    /// Start a paths download builder.
    fn paths(&self, snapshot: &str, data_type: &str) -> PathsDownload {
        PathsDownload {
            client: self.clone(),
            snapshot: snapshot.to_string(),
            data_type: data_type.to_string(),
        }
    }

    /// Start a data download builder.
    fn download(&self, path_file: &str) -> DataDownload {
        DataDownload {
            client: self.clone(),
            path_file: PathBuf::from(path_file),
            files_only: false,
            numbered: false,
            strict: false,
        }
    }
}

/// Builder for downloading path index files.
#[pyclass]
pub struct PathsDownload {
    client: Client,
    snapshot: String,
    data_type: String,
}

#[pymethods]
impl PathsDownload {
    /// Execute the download, saving to `dst`.
    fn to(&self, dst: &str) -> PyResult<()> {
        let dst_path = Path::new(dst);
        std::fs::create_dir_all(dst_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

        let options = download::DownloadOptions {
            snapshot: self.snapshot.clone(),
            data_type: &self.data_type,
            dst: dst_path,
            max_retries: self.client.retries,
            progress: self.client.progress,
            ..Default::default()
        };

        run_async(download::download_paths(options))
    }
}

/// Builder for downloading data files.
#[pyclass]
pub struct DataDownload {
    client: Client,
    path_file: PathBuf,
    files_only: bool,
    numbered: bool,
    strict: bool,
}

#[pymethods]
impl DataDownload {
    /// Download files without folder structure.
    fn files_only(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.files_only = true;
        slf
    }

    /// Enumerate output files (for Ungoliant compatibility).
    fn numbered(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.numbered = true;
        slf
    }

    /// Abort on unrecoverable errors (401, 403, 404).
    fn strict(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.strict = true;
        slf
    }

    /// Execute the download, saving to `dst`.
    fn to(&self, dst: &str) -> PyResult<()> {
        let dst_path = Path::new(dst);
        std::fs::create_dir_all(dst_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

        let options = download::DownloadOptions {
            paths: &self.path_file,
            dst: dst_path,
            threads: self.client.threads,
            max_retries: self.client.retries,
            numbered: self.numbered,
            files_only: self.files_only,
            progress: self.client.progress,
            strict: self.strict,
            ..Default::default()
        };

        run_async(download::download(options))
    }
}

/// ccdown Python module
#[pymodule]
pub fn ccdown(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<PathsDownload>()?;
    m.add_class::<DataDownload>()?;
    Ok(())
}
