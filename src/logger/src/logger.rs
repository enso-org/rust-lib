//! Contains implementation of default logger.

use enso_prelude::*;

use crate::Message;
use crate::LoggerOps;
use crate::level;
use crate::level::Levels;

use enso_shapely::CloneRef;
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::console;



// ==============
// === Logger ===
// ==============

// /// Default Logger implementation.
// #[cfg(not(target_arch="wasm32"))]
// pub type Logger = NativeLogger;

/// Default Logger implementation.
// #[cfg(target_arch="wasm32")]



// ====================
// === NativeLogger ===
// ====================
//
// /// Native logger implementation.
// #[derive(Clone,CloneRef,Debug,Default)]
// pub struct NativeLogger {
//     /// Path that is used as an unique identifier of this logger.
//     path         : ImString,
//     trace_copies : TraceCopies,
//     indent       : Rc<Cell<usize>>,
// }
//
// impl NativeLogger {
//     fn format(&self, msg:impl Message) -> String {
//         let indent = " ".repeat(4*self.indent.get());
//         iformat!("{indent}[{self.path}] {msg.get()}")
//     }
//
//     fn inc_indent(&self) {
//         self.indent.update(|t|t.saturating_add(1));
//     }
//
//     fn dec_indent(&self) {
//         self.indent.update(|t|t.saturating_sub(1));
//     }
// }
//
//
//
// impl AnyLogger for NativeLogger {
//     type Owned = Self;
//     type Level = Level; // FIXME
//     fn new(path:impl Into<ImString>) -> Self {
//         let path         = path.into();
//         let indent       = default();
//         let trace_copies = default();
//         Self {path,indent,trace_copies}
//     }
//     fn path        (&self) -> &str           { &self.path }
//     fn trace       (&self, msg:impl Message) { println!("{}",self.format(msg)) }
//     fn debug       (&self, msg:impl Message) { println!("{}",self.format(msg)) }
//     fn info        (&self, msg:impl Message) { println!("{}",self.format(msg)) }
//     fn warning     (&self, msg:impl Message) { println!("[WARNING] {}",self.format(msg)) }
//     fn error       (&self, msg:impl Message) { println!("[ERROR] {}",self.format(msg)) }
//     fn group_begin (&self, msg:impl Message) { println!("{}",self.format(msg)); self.inc_indent() }
//     fn trace_copies(&self)                   { self.trace_copies.enable(&self.path) }
//     fn group_end   (&self)                   { self.dec_indent() }
//     fn warning_group_end (&self)             { self.dec_indent() }
//     fn error_group_end   (&self)             { self.dec_indent() }
//     fn warning_group_begin (&self, msg:impl Message) {
//         println!("[WARNING] {}",self.format(msg)); self.inc_indent()
//     }
//     fn error_group_begin (&self, msg:impl Message) {
//         println!("[ERROR] {}",self.format(msg)); self.inc_indent()
//     }
// }

#[derive(Debug)]
pub struct Entry {
    pub message : String,
}

impl Entry {
    pub fn new(message:impl Message) -> Self {
        let message = message.get();
        Self {message}
    }
}

#[derive(Debug)]
pub struct Event<Level> {
    level : Level,
    tp    : EventType,
}

#[derive(Debug)]
pub enum EventType {
    Entry(Entry),
    GroupBegin(Entry),
    GroupEnd
}

impl<Level> Event<Level> {
    pub fn entry(level:impl Into<Level>, message:impl Message) -> Self {
        let level = level.into();
        let tp    = EventType::Entry(Entry::new(message));
        Self {level,tp}
    }

    pub fn group_begin(level:impl Into<Level>, message:impl Message) -> Self {
        let level = level.into();
        let tp    = EventType::GroupBegin(Entry::new(message));
        Self {level,tp}
    }

    pub fn group_end(level:impl Into<Level>) -> Self {
        let level = level.into();
        let tp    = EventType::GroupEnd;
        Self {level,tp}
    }

    pub fn casted<L>(self, level:L) -> Event<L> {
        let tp = self.tp;
        Event{level,tp}
    }
}

#[derive(Debug,Default)]
pub struct DefaultSink;

impl DefaultSink {
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

pub trait Sink<Level> : Default {
    fn submit(&mut self, path:&str, event:Event<Level>);
}

// impl<Level> Sink<Level> for DefaultSink {
//     default fn submit(&self, path:&str, event:Event<Level>) {}
// }Trace,Debug,Info,Warning,Error

impl Sink<level::Levels> for DefaultSink {
    default fn submit(&mut self, path:&str, event:Event<level::Levels>) {
        match event.level {
            // level::Levels::Trace   => self.submit(path,event.casted(level::Trace)),
            // level::Levels::Debug   => self.submit(path,event.casted(level::Debug)),
            // level::Levels::Info    => self.submit(path,event.casted(level::Info)),
            level::Levels::Warning => self.submit(path,event.casted(level::Warning)),
            // level::Levels::Error   => self.submit(path,event.casted(level::Error)),
            _ => {} // FIXME
        }
    }
}


impl Sink<level::Warning> for DefaultSink {
    default fn submit(&mut self, path:&str, event:Event<level::Warning>) {
        match event.tp {
            EventType::Entry(entry) => {
                let args = self.format_warn(path,entry.message);
                console::log_3 (&args.0,&args.1,&args.2)
            },
            EventType::GroupBegin(entry) => {
                let args = self.format_warn(path,entry.message);
                console::group_collapsed_3 (&args.0,&args.1,&args.2)
            },
            EventType::GroupEnd => {
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
pub struct Logger<Filter=level::from::Trace, Sink=DefaultSink, Level=Levels> {
    path   : ImString,
    filter : PhantomData<Filter>,
    sink   : Rc<RefCell<Sink>>,
}

impl<Filter,S,Level> Logger<Filter,S,Level>
where S:Sink<Level> {
    pub fn new(path:impl Into<ImString>) -> Self {
        let path   = path.into();
        let filter = default();
        let sink   = default();
        Self {path,filter,sink}
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


/// Interface common to all loggers.
pub trait AnyLogger {
    /// Path that is used as an unique identifier of this logger.
    fn path(&self) -> &str;
}

impl<T:AnyLogger> AnyLogger for &T {
    fn path(&self) -> &str { T::path(self) }
}

impl<Filter,Sink,Level> AnyLogger for Logger<Filter,Sink,Level> {
    fn path        (&self) -> &str           { &self.path }
    // fn trace       (&self, msg:impl Message) { console::trace_1 (&self.format(msg)) }
    // fn debug       (&self, msg:impl Message) { console::debug_1 (&self.format(msg)) }
    // fn info        (&self, msg:impl Message) { console::info_1  (&self.format(msg)) }
    // fn warning     (&self, msg:impl Message) {
    //     let args = self.format_warn(msg);
    //     console::log_3 (&args.0,&args.1,&args.2)
    // }
    // fn error       (&self, msg:impl Message) {
    //     let args = self.format_err(msg);
    //     console::log_3 (&args.0,&args.1,&args.2)
    // }
    // fn group_begin (&self, msg:impl Message) { console::group_1 (&self.format(msg)) }
    // fn trace_copies(&self)                   { self.trace_copies.enable(&self.path) }
    // fn group_end         (&self)             { console::group_end() }
    // fn warning_group_end (&self)             { console::group_end() }
    // fn error_group_end   (&self)             { console::group_end() }
    // fn warning_group_begin (&self, msg:impl Message) {
    //     let args = self.format_warn(msg);
    //     console::group_collapsed_3 (&args.0,&args.1,&args.2)
    // }
    // fn error_group_begin (&self, msg:impl Message) {
    //     let args = self.format_err(msg);
    //     console::group_collapsed_3 (&args.0,&args.1,&args.2)
    // }
}

// impl<S,Level:From<L>,L> LoggerOps<L> for Logger<level::from::Trace,S,Level>
// where S:Sink<Level> {
//     fn log(&self, level:L, msg:impl Message) {
//         println!("log0");
//         // self.entries.borrow_mut().push(Event::entry(level,msg))
//         self.sink.borrow_mut().submit(&self.path,Event::entry(level,msg))
//     }
//
//     fn group_begin(&self, level:L, collapsed:bool, msg:impl Message) {
//         self.entries.borrow_mut().push(Event::group_begin(level,msg))
//     }
//
//     fn group_end(&self, level:L) {
//         self.entries.borrow_mut().push(Event::group_end(level))
//     }
// }


// ======================
// === Logger Aliases ===
// ======================

macro_rules! define_logger_aliases {
    ($($tp:ident $name:ident $default_name:ident;)*) => {$(
        /// A logger which compile-time filters out all messages with log levels smaller than $tp.
        pub type $name <S=DefaultSink,L=Levels> = Logger<level::from::$tp,S,L>;

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
    where S:Sink<Level>, Level:From<L> {
    default fn log(&self, level:L, msg:impl Message) {
        self.sink.borrow_mut().submit(&self.path,Event::entry(level,msg))
    }

    default fn group_begin(&self, level:L, collapsed:bool, msg:impl Message) {
        self.sink.borrow_mut().submit(&self.path,Event::group_begin(level,msg))
    }

    default fn group_end(&self, level:L) {
        self.sink.borrow_mut().submit(&self.path,Event::group_end(level))
    }
}


// === Compile-Time Filtering ===

macro_rules! disable_logger {
    ($(level::from::$filter:ident for $($level:ident),*;)*) => {$($(
        impl<S,Level> LoggerOps<level::$level> for Logger<level::from::$filter,S,Level>
        where S:Sink<Level>, Level:From<level::$level> {}
    )*)*};
}

disable_logger! {
    level::from::Debug   for Trace;
    level::from::Info    for Trace,Debug;
    level::from::Warning for Trace,Debug,Info;
    level::from::Error   for Trace,Debug,Info,Warning;
}
