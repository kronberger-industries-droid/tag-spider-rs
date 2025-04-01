// src/error.rs

use thirtyfour::error::WebDriverError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyLibraryError {
    #[error("WebDriver error: {0}")]
    WebDriver(#[from] WebDriverError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Custom error: {0}")]
    Custom(String),
}
