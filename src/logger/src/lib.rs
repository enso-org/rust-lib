//! This crate contains implementation of logging interface.

#![feature(cell_update)]

#![deny(unconditional_recursion)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![feature(specialization)]



pub mod level;
pub mod logger;
pub mod macros;
pub mod message;
pub mod ops;

pub use logger::Logger;
pub use logger::AnyLogger;
pub use logger::TraceLogger;
pub use logger::DebugLogger;
pub use logger::InfoLogger;
pub use logger::WarningLogger;
pub use logger::ErrorLogger;
pub use logger::DefaultTraceLogger;
pub use logger::DefaultDebugLogger;
pub use logger::DefaultInfoLogger;
pub use logger::DefaultWarningLogger;
pub use logger::DefaultErrorLogger;

pub use message::Message;
pub use ops::LoggerOps;
