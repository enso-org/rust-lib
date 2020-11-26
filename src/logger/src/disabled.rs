//! Contains definition of trivial logger that discards all messages except warnings and errors.

use enso_prelude::*;

use crate::Message;
use crate::enabled::AnyLogger;
use crate::enabled;
use crate::enabled::Event;
use crate::level;
use crate::level::Level;
use crate::Log;
use crate::Group;

use enso_shapely::CloneRef;
use std::fmt::Debug;



// ==============
// === Logger ===
// ==============

pub type Logger = enabled::Logger<level::from::Warning,Level>;
