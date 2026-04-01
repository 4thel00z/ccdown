use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Download paths for a given crawl
    DownloadPaths {
        /// Crawl reference, e.g. CC-MAIN-2021-04 or CC-NEWS-2025-01
        #[arg(value_name = "CRAWL", value_parser = crawl_name_format)]
        snapshot: String,

        /// Data type
        #[arg(value_name = "SUBSET")]
        data_type: DataType,

        /// Destination folder
        #[arg(value_name = "DESTINATION")]
        dst: PathBuf,
    },

    /// Download files from a crawl
    Download {
        /// Path file
        #[arg(value_name = "PATHS")]
        path_file: PathBuf,

        /// Destination folder
        #[arg(value_name = "DESTINATION")]
        dst: PathBuf,

        /// Download files without the folder structure. This only works for WARC/WET/WAT files
        #[arg(short, long)]
        files_only: bool,

        ///Enumerate output files for compatibility with Ungoliant Pipeline. This only works for WET files
        #[arg(short, long)]
        numbered: bool,

        /// Number of threads to use
        #[arg(short, long, default_value = "10", value_name = "NUMBER OF THREADS")]
        threads: usize,

        /// Maximum number of retries per file
        #[arg(
            short,
            long,
            default_value = "1000",
            value_name = "MAX RETRIES PER FILE"
        )]
        retries: usize,

        /// Print progress
        #[arg(short, long, action)]
        progress: bool,

        /// Abort all downloads on unrecoverable errors (401, 403, 404)
        #[arg(short, long)]
        strict: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DataType {
    Segment,
    Warc,
    Wat,
    Wet,
    Robotstxt,
    Non200responses,
    CcIndex,
    CcIndexTable,
}

impl DataType {
    pub fn as_str(&self) -> &str {
        match self {
            DataType::Segment => "segment",
            DataType::Warc => "warc",
            DataType::Wat => "wat",
            DataType::Wet => "wet",
            DataType::Robotstxt => "robotstxt",
            DataType::Non200responses => "non200responses",
            DataType::CcIndex => "cc-index",
            DataType::CcIndexTable => "cc-index-table",
        }
    }
}

pub fn crawl_name_format(crawl: &str) -> Result<String, String> {
    let main_re = Regex::new(r"^(CC\-MAIN)\-([0-9]{4})\-([0-9]{2})$").unwrap();
    let news_re = Regex::new(r"^(CC\-NEWS)\-([0-9]{4})\-([0-9]{2})$").unwrap();

    let crawl_ref = crawl.to_uppercase();

    if !(main_re.is_match(&crawl_ref) || news_re.is_match(&crawl_ref)) {
        Err("Please use the CC-MAIN-YYYY-WW or the CC-NEWS-YYYY-MM format.".to_string())
    } else {
        Ok(crawl_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // crawl_name_format tests
    #[test]
    fn valid_cc_main_format() {
        assert_eq!(
            crawl_name_format("CC-MAIN-2025-08"),
            Ok("CC-MAIN-2025-08".to_string())
        );
    }

    #[test]
    fn valid_cc_news_format() {
        assert_eq!(
            crawl_name_format("CC-NEWS-2025-01"),
            Ok("CC-NEWS-2025-01".to_string())
        );
    }

    #[test]
    fn case_insensitive_crawl_name() {
        assert_eq!(
            crawl_name_format("cc-main-2021-04"),
            Ok("CC-MAIN-2021-04".to_string())
        );
    }

    #[test]
    fn invalid_crawl_name_missing_week() {
        assert!(crawl_name_format("CC-MAIN-2025").is_err());
    }

    #[test]
    fn invalid_crawl_name_wrong_prefix() {
        assert!(crawl_name_format("CC-OTHER-2025-08").is_err());
    }

    #[test]
    fn invalid_crawl_name_extra_digits() {
        assert!(crawl_name_format("CC-MAIN-2025-123").is_err());
    }

    #[test]
    fn invalid_crawl_name_empty() {
        assert!(crawl_name_format("").is_err());
    }

    // DataType::as_str tests
    #[test]
    fn data_type_segment() {
        assert_eq!(DataType::Segment.as_str(), "segment");
    }

    #[test]
    fn data_type_warc() {
        assert_eq!(DataType::Warc.as_str(), "warc");
    }

    #[test]
    fn data_type_wat() {
        assert_eq!(DataType::Wat.as_str(), "wat");
    }

    #[test]
    fn data_type_wet() {
        assert_eq!(DataType::Wet.as_str(), "wet");
    }

    #[test]
    fn data_type_robotstxt() {
        assert_eq!(DataType::Robotstxt.as_str(), "robotstxt");
    }

    #[test]
    fn data_type_non200responses() {
        assert_eq!(DataType::Non200responses.as_str(), "non200responses");
    }

    #[test]
    fn data_type_cc_index() {
        assert_eq!(DataType::CcIndex.as_str(), "cc-index");
    }

    #[test]
    fn data_type_cc_index_table() {
        assert_eq!(DataType::CcIndexTable.as_str(), "cc-index-table");
    }
}
