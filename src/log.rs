use std::fmt;

use colored::Colorize;

/// Enum used to display messages with their correspondant log level
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogLevel::Info => "[INFO]".bold().cyan(),
            LogLevel::Warn => "[WARN]".bold().yellow(),
            LogLevel::Error => "[ERROR]".bold().red(),
        };
        write!(f, "{}", s)
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("\n{} {}", crate::log::LogLevel::Info, format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("\n{} {}", crate::log::LogLevel::Warn, format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("\n{} {}", crate::log::LogLevel::Error, format!($($arg)*));
    };
}
