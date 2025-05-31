//! Struct used for mapping various libraries errors.

use core::fmt;

/// Contains a message to be displayed in case the app terminates prematurely
/// (or for debugging purposes).
#[derive(Debug)]
pub struct AppError {
    pub msg: String,
}

impl AppError {
    pub fn new(msg: impl Into<String>) -> AppError {
        AppError { msg: msg.into() }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
