pub mod cli;
pub mod download;
pub mod errors;

#[cfg(feature = "python")]
pub mod python;

pub use download::{download, download_paths, DownloadOptions};
pub use errors::DownloadError;
