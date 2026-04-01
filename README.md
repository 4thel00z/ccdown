<p align="center">
  <img src="logo.png" alt="ccdown" width="500">
</p>

<p align="center">
  A polite downloader for <a href="https://commoncrawl.org">Common Crawl</a> data, written in Rust.
</p>

<p align="center">
  <a href="https://crates.io/crates/ccdown"><img src="https://img.shields.io/crates/v/ccdown.svg" alt="crates.io"></a>
  <a href="https://docs.rs/ccdown"><img src="https://docs.rs/ccdown/badge.svg" alt="docs.rs"></a>
  <a href="https://github.com/4thel00z/ccdown/actions"><img src="https://github.com/4thel00z/ccdown/workflows/Rust/badge.svg" alt="CI"></a>
  <a href="https://github.com/4thel00z/ccdown/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
</p>

---

## Install

```bash
cargo install ccdown
```

<details>
<summary>Other methods</summary>

### From source

```bash
git clone https://github.com/4thel00z/ccdown.git
cd ccdown
cargo install --path .
```

### Pre-built binaries

Grab the latest release for your platform from the [releases page](https://github.com/4thel00z/ccdown/releases).

</details>

## Usage

### 1. Download the path manifest for a crawl

```bash
ccdown download-paths CC-MAIN-2025-08 warc ./paths
```

Supported subsets: `segment` `warc` `wat` `wet` `robotstxt` `non200responses` `cc-index` `cc-index-table`

Crawl format: `CC-MAIN-YYYY-WW` or `CC-NEWS-YYYY-MM`

### 2. Download the actual data

```bash
ccdown download ./paths/warc.paths.gz ./data
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t` | Number of concurrent downloads | `10` |
| `-r` | Max retries per file | `1000` |
| `-p` | Show progress bars | off |
| `-f` | Flat file output (no directory structure) | off |
| `-n` | Numbered output (for Ungoliant Pipeline) | off |

### Example

```bash
ccdown download -p -t 5 ./paths/warc.paths.gz ./data
```

> **Note:** Keep threads at 10 or below. Too many concurrent requests will get you `403`'d by the server, and those errors are unrecoverable.

## License

MIT OR Apache-2.0
