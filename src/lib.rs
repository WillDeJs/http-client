//! A HTTP Client Library.
//!
//! # Example 1:
//! ``` no_run
//! use http_client::{client::Client, error::HttpError};
//! fn main() -> Result<(), HttpError> {
//!     let client = Client::new();
//!     client
//!         .post("localhost:8080/login_action")?
//!         .form_data("email", "test@mail.com")
//!         .form_data("password", "password")
//!         .send()?;
//!     Ok(())
//! }
//! ```
//! # Example 2:
//! ``` no_run
//! use http_client::{client::Client, error::HttpError};
//! fn main() -> Result<(), HttpError> {
//!     let mut file = std::fs::File::create("video.mp4")?;
//!     let client = Client::new();
//!     client
//!         .get("localhost:8080/Video.mp4")?
//!         .download_to_file(&mut file)?;
//!     Ok(())
//! }
//! ```
//!
#[allow(dead_code)]
pub mod client;
mod config;
#[allow(dead_code)]
pub mod error;

#[allow(dead_code)]
pub mod json;

pub use http_parse::*;
