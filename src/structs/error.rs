//! Struct used for mapping various libraries errors.

use core::fmt;

/// Contains a message to be displayed in case the app terminates prematurely
/// (or for debugging purposes).
#[derive(Debug)]
pub struct AppError {
    pub msg: String,
}

impl AppError {
    pub fn new(msg: fmt::Arguments) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }

    pub fn cancelled() -> Self {
        Self {
            msg: "Task cancelled by user".to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

/// Simplifies the struct creation.
#[macro_export]
macro_rules! app_error {
    ($($arg:tt)*) => {
        AppError::new(format_args!($($arg)*))
    };
}
