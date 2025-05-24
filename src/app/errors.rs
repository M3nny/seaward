use core::fmt;

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
