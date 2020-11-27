//! Contains implementation of default logger.


use crate::prelude::*;

use crate::Message;
use crate::entry::level;
use crate::entry::level::DefaultLevels;

use enso_shapely::CloneRef;
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::console;

use crate::entry::Entry;
use crate::entry::EntryContent;

use crate::sink::formatter;
use crate::sink::formatter::Formatter;
use crate::sink::formatter::FormatterOutput;
use crate::sink::consumer;
use crate::sink::consumer::Consumer;
use crate::sink::LevelGroupSink;
use crate::sink::Sink;




impl<S,Fmt> LevelGroupSink<level::DefaultLevels> for Sink<S,Fmt>
where S:Consumer<level::DefaultLevels,Fmt::Output>,
      Fmt:Formatter<level::Warning>,
      Fmt:Formatter<level::Debug>,
{
    fn submit(&mut self, path:&str, event:Entry<level::DefaultLevels>) {
        match event.level {
            level::DefaultLevels::Debug => {
                let msg = formatter::format::<Fmt,level::Debug>(path,&event.content);
                self.consumer.consume(event,msg);
            },
            level::DefaultLevels::Warning => {
                let msg = formatter::format::<Fmt,level::Warning>(path,&event.content);
                self.consumer.consume(event,msg);
            },
            _ => {} // FIXME
        }
    }
}




// ==============
// === Logger ===
// ==============

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Logger<Filter=level::from::Trace, S=Sink, Level=DefaultLevels> {
    path   : ImString,
    filter : PhantomData<Filter>,
    levels : PhantomData<Level>,
    sink   : Rc<RefCell<S>>,
}

impl<Filter,S,Level> Logger<Filter,S,Level>
where S:LevelGroupSink<Level>+Default {
    pub fn new(path:impl Into<ImString>) -> Self {
        let path   = path.into();
        let filter = default();
        let levels = default();
        let sink   = default();
        Self {path,filter,levels,sink}
    }

    fn format(&self, msg:impl Message) -> JsValue {
        iformat!("[{self.path}] {msg.get()}").into()
    }

    fn format_color(&self, color:&str, msg:String) -> (JsValue,JsValue,JsValue) {
        let msg  = format!("%c {} %c {}",self.path,msg).into();
        let css1 = "background:dimgray;border-radius:4px".into();
        let css2 = format!("color:{}",color).into();
        (msg,css1,css2)
    }

    fn format_warn(&self, msg:impl Message) -> (JsValue,JsValue,JsValue) {
        self.format_color("orange",format!("[W] {}",msg.get()))
    }

    fn format_err(&self, msg:impl Message) -> (JsValue,JsValue,JsValue) {
        self.format_color("orangered",format!("[E] {}",msg.get()))
    }

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

// A common interface for all loggers. Exposing all information needed to create a particular
// sub-logger from a given parent logger of any type.
pub trait AnyLogger {
    /// Path that is used as an unique identifier of this logger.
    fn path(&self) -> &str;
}

impl<T:AnyLogger> AnyLogger for &T {
    fn path(&self) -> &str { T::path(self) }
}

impl<Filter,Sink,Level> AnyLogger for Logger<Filter,Sink,Level> {
    fn path (&self) -> &str { &self.path }
}



// ======================
// === Logger Aliases ===
// ======================

macro_rules! define_logger_aliases {
    ($($tp:ident $name:ident $default_name:ident;)*) => {$(
        /// A logger which compile-time filters out all messages with log levels smaller than $tp.
        pub type $name <S=Sink,L=DefaultLevels> = Logger<level::from::$tp,S,L>;

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
where S:LevelGroupSink<Level>, Level:From<L> {
    default fn log(&self, level:L, msg:impl Message) {
        self.sink.borrow_mut().submit(&self.path,Entry::message(level,msg))
    }

    default fn group_begin(&self, level:L, collapsed:bool, msg:impl Message) {
        self.sink.borrow_mut().submit(&self.path,Entry::group_begin(level,msg,collapsed))
    }

    default fn group_end(&self, level:L) {
        self.sink.borrow_mut().submit(&self.path,Entry::group_end(level))
    }
}


// === Compile-Time Filtering ===

macro_rules! disable_logger {
    ($(level::from::$filter:ident for $($level:ident),*;)*) => {$($(
        impl<S,Level> LoggerOps<level::$level> for Logger<level::from::$filter,S,Level>
        where S:LevelGroupSink<Level>, Level:From<level::$level> {
            fn log(&self, _level:level::$level, _msg:impl Message) {}
            fn group_begin(&self, _level:level::$level, _collapsed:bool, _msg:impl Message) {}
            fn group_end(&self, _level:level::$level) {}
        }
    )*)*};
}

disable_logger! {
    level::from::Debug   for Trace;
    level::from::Info    for Trace,Debug;
    level::from::Warning for Trace,Debug,Info;
    level::from::Error   for Trace,Debug,Info,Warning;
}
