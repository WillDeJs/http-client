use std::{fmt::Display, num::ParseIntError};

use http_parse::HttpParseError;

#[derive(Debug)]
pub enum HttpError {
    BadResponse(usize, String),
    InvalidUrl(String),
    ParseError(String),
    ConnectionError(String),
}

impl core::error::Error for HttpError {}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::BadResponse(status_code, message) => write!(f, "{status_code}: {message}"),
            HttpError::ParseError(e) => write!(f, "{e}"),
            HttpError::InvalidUrl(http_url) => write!(f, "Invalid Url: `{http_url}`"),
            HttpError::ConnectionError(e) => write!(f, "Connection error: `{e}`"),
        }
    }
}

impl From<ParseIntError> for HttpError {
    fn from(value: ParseIntError) -> Self {
        HttpError::ParseError(value.to_string())
    }
}

impl From<HttpParseError> for HttpError {
    fn from(value: HttpParseError) -> Self {
        HttpError::ParseError(value.to_string())
    }
}

impl From<std::io::Error> for HttpError {
    fn from(value: std::io::Error) -> Self {
        HttpParseError::from(value).into()
    }
}
