//! Contains definition of trivial logger that discards all messages except warnings and errors.

use enso_prelude::*;

use crate::Message;
use crate::logger::AnyLogger;
use crate::logger;
use crate::logger::Event;
use crate::level;
use crate::level::Level;

use enso_shapely::CloneRef;
use std::fmt::Debug;



// ==============
// === Logger ===
// ==============

pub type Logger = logger::Logger<level::from::Warning,Level>;
