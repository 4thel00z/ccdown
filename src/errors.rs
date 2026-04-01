use std::fmt;

#[derive(Debug)]
pub enum DownloadError {
    Reqwest(reqwest::Error),
    ReqwestMiddleware(reqwest_middleware::Error),
    Tokio(tokio::io::Error),
    Url(url::ParseError),
    Indicatif(indicatif::style::TemplateError),
    Join(tokio::task::JoinError),
    Custom(String),
    Unrecoverable(u16, String),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DownloadError::Reqwest(ref err) => err.fmt(f),
            DownloadError::ReqwestMiddleware(ref err) => err.fmt(f),
            DownloadError::Tokio(ref err) => err.fmt(f),
            DownloadError::Url(ref err) => err.fmt(f),
            DownloadError::Indicatif(ref err) => err.fmt(f),
            DownloadError::Join(ref err) => err.fmt(f),
            DownloadError::Custom(ref err) => err.fmt(f),
            DownloadError::Unrecoverable(status, ref url) => {
                write!(f, "Unrecoverable error (HTTP {}): {}", status, url)
            }
        }
    }
}

impl DownloadError {
    pub fn is_unrecoverable(&self) -> bool {
        matches!(self, DownloadError::Unrecoverable(401 | 403 | 404, _))
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::Reqwest(err)
    }
}

impl From<reqwest_middleware::Error> for DownloadError {
    fn from(err: reqwest_middleware::Error) -> Self {
        DownloadError::ReqwestMiddleware(err)
    }
}

impl From<tokio::io::Error> for DownloadError {
    fn from(err: tokio::io::Error) -> Self {
        DownloadError::Tokio(err)
    }
}

impl From<url::ParseError> for DownloadError {
    fn from(err: url::ParseError) -> Self {
        DownloadError::Url(err)
    }
}

impl From<indicatif::style::TemplateError> for DownloadError {
    fn from(err: indicatif::style::TemplateError) -> Self {
        DownloadError::Indicatif(err)
    }
}

impl From<tokio::task::JoinError> for DownloadError {
    fn from(err: tokio::task::JoinError) -> Self {
        DownloadError::Join(err)
    }
}

impl From<String> for DownloadError {
    fn from(s: String) -> DownloadError {
        DownloadError::Custom(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_custom() {
        let err = DownloadError::Custom("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn display_unrecoverable() {
        let err = DownloadError::Unrecoverable(404, "https://example.com/file".to_string());
        assert_eq!(
            err.to_string(),
            "Unrecoverable error (HTTP 404): https://example.com/file"
        );
    }

    #[test]
    fn display_url_error() {
        let err: DownloadError = url::ParseError::EmptyHost.into();
        assert!(err.to_string().contains("empty host"));
    }

    #[test]
    fn display_tokio_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: DownloadError = io_err.into();
        assert_eq!(err.to_string(), "file not found");
    }

    #[test]
    fn from_string() {
        let err: DownloadError = "test error".to_string().into();
        assert!(matches!(err, DownloadError::Custom(s) if s == "test error"));
    }

    #[test]
    fn from_url_parse_error() {
        let err: DownloadError = url::ParseError::EmptyHost.into();
        assert!(matches!(err, DownloadError::Url(_)));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io fail");
        let err: DownloadError = io_err.into();
        assert!(matches!(err, DownloadError::Tokio(_)));
    }

    #[test]
    fn is_unrecoverable_401() {
        let err = DownloadError::Unrecoverable(401, "url".to_string());
        assert!(err.is_unrecoverable());
    }

    #[test]
    fn is_unrecoverable_403() {
        let err = DownloadError::Unrecoverable(403, "url".to_string());
        assert!(err.is_unrecoverable());
    }

    #[test]
    fn is_unrecoverable_404() {
        let err = DownloadError::Unrecoverable(404, "url".to_string());
        assert!(err.is_unrecoverable());
    }

    #[test]
    fn is_unrecoverable_false_for_500() {
        let err = DownloadError::Unrecoverable(500, "url".to_string());
        assert!(!err.is_unrecoverable());
    }

    #[test]
    fn is_unrecoverable_false_for_custom() {
        let err = DownloadError::Custom("not unrecoverable".to_string());
        assert!(!err.is_unrecoverable());
    }
}
