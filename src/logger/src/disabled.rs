//! Contains definition of trivial logger that discards all messages except warnings and errors.

use enso_prelude::*;

use crate::Message;
use crate::AnyLogger;
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

pub type Logger = LoggerOf<Level>;

/// Trivial logger that discards all messages except warnings and errors.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct LoggerOf<Level> {
    enabled : enabled::LoggerOf<Level>,
}


// === Impls ===

impls!{ From + &From <enabled::Logger> for Logger { |logger| Self::new(logger.path()) }}

impl<Level> AnyLogger for LoggerOf<Level> {
    type Owned = Self;
    type Level = Level;
    fn new (path:impl Into<ImString>) -> Self { Self {enabled : enabled::LoggerOf::new(path) } }
    fn path                (&self) -> &str           { self.enabled.path() }
}


impl<Level:From<level::Warning>> Log<level::Warning> for LoggerOf<Level> {
    fn log(&self, level:level::Warning, msg:impl Message) {
        self.enabled.log(level,msg)
    }
}

impl<Level:From<level::Error>> Log<level::Error> for LoggerOf<Level> {
    fn log(&self, level:level::Error, msg:impl Message) {
        self.enabled.log(level,msg)
    }
}

impl<L,Level:From<L>> Log<L> for LoggerOf<Level> {
    default fn log(&self, _level:L, _msg:impl Message) {}
}



impl<Level:From<level::Warning>> Group<level::Warning> for LoggerOf<Level> {
    fn group_begin(&self, level:level::Warning, collapsed:bool, msg:impl Message) {
        self.enabled.group_begin(level,collapsed,msg)
    }
    fn group_end(&self, level:level::Warning) {
        self.enabled.group_end(level)
    }
}


impl<L,Level:From<L>> Group<L> for LoggerOf<Level> {
    default fn group_begin(&self, _level:L, _collapsed:bool, _msg:impl Message) {}
    default fn group_end(&self, _level:L) {}
}