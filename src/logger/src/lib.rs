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

pub mod disabled;
pub mod enabled;
pub mod message;
pub mod level;
pub mod macros;
pub mod ops;

pub use enabled::AnyLogger;

use enso_prelude::*;





pub use message::Message;
pub use ops::LoggerOps;










