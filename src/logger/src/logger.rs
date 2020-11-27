//! Contains implementation of default logger.

use enso_prelude::*;

use crate::Message;
use crate::LoggerOps;
use crate::level;
use crate::level::DefaultLevels;

use enso_shapely::CloneRef;
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::console;

use crate::entry::Entry;
use crate::entry::EntryContent;

use crate::formatter;
use crate::formatter::Formatter;
use crate::formatter::FormatterOutput;
use crate::consumer;
use crate::consumer::Consumer;





#[derive(Debug,Derivative)]
#[derivative(Default(bound="Consumer:Default"))]
pub struct Sink<Consumer=consumer::Default,Formatter=formatter::Default> {
    formatter : PhantomData<Formatter>,
    consumer  : Consumer,
}

impl<Fmt> Sink<Fmt> {
    fn format_color(&self, path:&str, color:&str, msg:String) -> (JsValue,JsValue,JsValue) {
        let msg  = format!("%c {} %c {}",path,msg).into();
        let css1 = "background:dimgray;border-radius:4px".into();
        let css2 = format!("color:{}",color).into();
        (msg,css1,css2)
    }

    fn format_warn(&self, path:&str, msg:impl Message) -> (JsValue,JsValue,JsValue) {
        self.format_color(path,"orange",format!("[W] {}",msg.get()))
    }
}

pub trait LevelGroupSink<Level> {
    fn submit(&mut self, path:&str, event:Entry<Level>);
}

pub trait LevelSink<Level> {
    fn submit(&mut self, path:&str, event:Entry<Level>);
}

// impl<Level> Sink<Level> for Sink {
//     default fn submit(&self, path:&str, event:Entry<Level>) {}
// }Trace,Debug,Info,Warning,Error

impl<S,Fmt> LevelGroupSink<level::DefaultLevels> for Sink<S,Fmt>
where S:Consumer<level::DefaultLevels,Fmt::Output>,
      Fmt:Formatter<level::Warning>,
      Fmt:Formatter<level::Debug>,
{
    fn submit(&mut self, path:&str, event:Entry<level::DefaultLevels>) {
        match event.level {
            // level::DefaultLevels::Trace   => self.submit(path,event.casted(level::Trace)),
            // level::DefaultLevels::Debug   => {
            //     let msg = <Fmt>::format(path,&event.content);
            //     self.consumer.consume(event,msg);
            // },
            // level::DefaultLevels::Info    => self.submit(path,event.casted(level::Info)),
            level::DefaultLevels::Debug => {
                let msg = formatter::format::<Fmt,level::Debug>(path,&event.content);
                self.consumer.consume(event,msg);
            },
            level::DefaultLevels::Warning => {
                let msg = formatter::format::<Fmt,level::Warning>(path,&event.content);
                self.consumer.consume(event,msg);
            },
            // level::DefaultLevels::Error   => self.submit(path,event.casted(level::Error)),
            _ => {} // FIXME
        }
    }
}


// pub trait Formatter<Level> : FormatterOutput {
//     fn format(path:&str, message:String) -> Self::Output;
// }

impl<S,Fmt,Level> LevelSink<Level> for Sink<S,Fmt>
where Fmt:Formatter<Level,Output=js_sys::Array> {
    default fn submit(&mut self, path:&str, event:Entry<Level>) {
        match event.content {
            EntryContent::Message(_) => {
                if let Some(msg) = <Fmt>::format(path,&event.content) {
                    console::log(&msg);
                }
            },
            EntryContent::GroupBegin(_) => {
                if let Some(msg) = <Fmt>::format(path,&event.content) {
                    console::group_collapsed(&msg);
                }
            },
            EntryContent::GroupEnd => {
                console::group_end()
            }
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



// ===============================
// === Ops to Sink Redirection ===
// ===============================

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
