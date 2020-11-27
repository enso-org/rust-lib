//! Extensible logger implementation.

#![deny(unconditional_recursion)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![feature(specialization)]

pub mod entry;
pub mod macros;
pub mod processor;

pub use enso_prelude as prelude;
pub use entry::message::Message;

use crate::entry::Entry;
use crate::entry::DefaultFilter;
use crate::entry::DefaultLevels;
use crate::processor::DefaultProcessor;
use crate::processor::Processor;

use prelude::*;

use enso_shapely::CloneRef;
use std::fmt::Debug;



// ==============
// === Logger ===
// ==============

/// The main logger implementation.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Logger<Filter=DefaultFilter, Processor=DefaultProcessor, Levels=DefaultLevels> {
    path   : ImString,
    filter : PhantomData<Filter>,
    levels : PhantomData<Levels>,
    sink   : Rc<RefCell<Processor>>,
}

impl<Filter,S,Level> Logger<Filter,S,Level>
where S:Default {
    /// Constructor.
    pub fn new(path:impl Into<ImString>) -> Self {
        let path   = path.into();
        let filter = default();
        let levels = default();
        let sink   = default();
        Self {path,filter,levels,sink}
    }

    /// Constructor from another logger keeping the same path.
    pub fn new_from(logger:impl AnyLogger) -> Self {
        Self::new(logger.path())
    }

    /// Creates a new logger with this logger as a parent.
    pub fn sub(logger:impl AnyLogger, id:impl AsRef<str>) -> Self {
        Self::new(iformat!("{logger.path()}.{id.as_ref()}"))
    }
}



// =================
// === AnyLogger ===
// =================

/// A common interface for all loggers. Exposing all information needed to create a particular
/// sub-logger from a given parent logger of any type.
pub trait AnyLogger {
    /// Path that is used as an unique identifier of this logger.
    fn path(&self) -> &str;
}

impl<T:AnyLogger> AnyLogger for &T {
    fn path(&self) -> &str { T::path(self) }
}

impl<Filter,Processor,Level> AnyLogger for Logger<Filter,Processor,Level> {
    fn path (&self) -> &str { &self.path }
}



// ======================
// === Logger Aliases ===
// ======================

macro_rules! define_logger_aliases {
    ($($tp:ident $name:ident $default_name:ident;)*) => {$(
        /// A logger which compile-time filters out all messages with log levels smaller than $tp.
        pub type $name <S=DefaultProcessor,L=DefaultLevels> = Logger<entry::filter_from::$tp,S,L>;

        /// The same as $name, but with all type arguments applied, for convenient usage.
        pub type $default_name = $name;
    )*};
}

define_logger_aliases! {
    Trace   TraceLogger   DefaultTraceLogger;
    Debug   DebugLogger   DefaultDebugLogger;
    Info    InfoLogger    DefaultInfoLogger;
    Warning WarningLogger DefaultWarningLogger;
    Error   ErrorLogger   DefaultErrorLogger;
}



// =================
// === LoggerOps ===
// =================

/// Primitive operations on a logger. The type parameter allows for compile-time log level filtering
/// of the messages.
#[allow(missing_docs)]
pub trait LoggerOps<Level> {
    fn log         (&self, level:Level, msg:impl Message);
    fn group_begin (&self, level:Level, collapsed:bool, msg:impl Message);
    fn group_end   (&self, level:Level);
}


// === Impl for References ===

impl<T:LoggerOps<Level>,Level> LoggerOps<Level> for &T {
    fn log(&self, level:Level, msg:impl Message) {
        LoggerOps::log(*self,level,msg)
    }

    fn group_begin(&self, level:Level, collapsed:bool, msg:impl Message) {
        LoggerOps::group_begin(*self,level,collapsed,msg)
    }

    fn group_end(&self, level:Level) {
        LoggerOps::group_end(*self,level)
    }
}


// === Generic Redirection ===

impl<S,Filter,Level,L> LoggerOps<L> for Logger<Filter,S,Level>
where S:Processor<Entry<Level>>, Level:From<L> {
    default fn log(&self, level:L, msg:impl Message) {
        self.sink.borrow_mut().submit(Entry::message(self.path.clone(),level,msg));
    }

    default fn group_begin(&self, level:L, collapsed:bool, msg:impl Message) {
        self.sink.borrow_mut().submit(Entry::group_begin(self.path.clone(),level,msg,collapsed));
    }

    default fn group_end(&self, level:L) {
        self.sink.borrow_mut().submit(Entry::group_end(self.path.clone(),level));
    }
}


// === Compile-time Filtering ===

/// Defines specialized version of compile time filtering rules for the given filtering levels.
/// It defines specialized implementations for the default implementation above. See the usage
/// below to learn more.
#[macro_export]
macro_rules! define_compile_time_filtering_rules {
    ($(level::from::$filter:ident for $($level:ident),*;)*) => {$($(
        impl<S,Level> LoggerOps<entry::level::$level>
        for Logger<entry::level::filter_from::$filter,S,Level>
        where S:Processor<Entry<Level>>, Level:From<entry::level::$level> {
            fn log         (&self, _lvl:entry::level::$level, _msg:impl Message) {}
            fn group_begin (&self, _lvl:entry::level::$level, _collapsed:bool, _msg:impl Message) {}
            fn group_end   (&self, _lvl:entry::level::$level) {}
        }
    )*)*};
}


// === Compile-time filtering of built-in levels ===

define_compile_time_filtering_rules! {
    level::from::Debug   for Trace;
    level::from::Info    for Trace,Debug;
    level::from::Warning for Trace,Debug,Info;
    level::from::Error   for Trace,Debug,Info,Warning;
}
