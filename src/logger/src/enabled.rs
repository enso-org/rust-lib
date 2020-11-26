//! Contains implementation of default logger.

use enso_prelude::*;

use crate::Message;
use crate::LoggerOps;
use crate::level;
use crate::level::Level;

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
pub struct Entry<Level> {
    pub level   : Level,
    pub message : String,
}

impl<Level> Entry<Level> {
    pub fn new(level:impl Into<Level>, message:impl Message) -> Self {
        let level   = level.into();
        let message = message.get();
        Self {level,message}
    }
}

#[derive(Debug)]
pub enum Event<Level> {
    Entry(Entry<Level>),
    GroupBegin(Entry<Level>),
    GroupEnd
}

impl<Level> Event<Level> {
    pub fn entry(level:impl Into<Level>, message:impl Message) -> Self {
        Self::Entry(Entry::new(level,message))
    }

    pub fn group_begin(level:impl Into<Level>, message:impl Message) -> Self {
        Self::GroupBegin(Entry::new(level,message))
    }

    pub fn group_end() -> Self {
        Self::GroupEnd
    }
}



// ==============
// === Logger ===
// ==============

pub type TraceLogger   <Level=level::Level> = Logger<level::from::Trace   , Level>;
pub type DebugLogger   <Level=level::Level> = Logger<level::from::Debug   , Level>;
pub type InfoLogger    <Level=level::Level> = Logger<level::from::Info    , Level>;
pub type WarningLogger <Level=level::Level> = Logger<level::from::Warning , Level>;
pub type ErrorLogger   <Level=level::Level> = Logger<level::from::Error   , Level>;

/// WASM logger implementation.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct Logger<Filter=level::from::Trace,Level=level::Level> {
    entries : Rc<RefCell<Vec<Event<Level>>>>,
    path    : ImString,
    filter  : PhantomData<Filter>,
}

impl<Filter,Level> Logger<Filter,Level> {
    pub fn new(path:impl Into<ImString>) -> Self {
        let path    = path.into();
        let entries = default();
        let filter  = default();
        Self {entries,path,filter}
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

impl<Filter,Level> AnyLogger for Logger<Filter,Level> {
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

impl<Level:From<L>,L> LoggerOps<L> for Logger<level::from::Trace,Level> {
    fn log(&self, level:L, msg:impl Message) {
        self.entries.borrow_mut().push(Event::entry(level,msg))
    }

    fn group_begin(&self, level:L, collapsed:bool, msg:impl Message) {
        self.entries.borrow_mut().push(Event::group_begin(level,msg))
    }

    fn group_end(&self, level:L) {
        self.entries.borrow_mut().push(Event::group_end())
    }
}





impl<Level:From<level::Warning>> LoggerOps<level::Warning> for Logger<level::from::Warning,Level> {
    fn log(&self, level:level::Warning, msg:impl Message) {
        self.entries.borrow_mut().push(Event::entry(level,msg))
    }

    fn group_begin(&self, level:level::Warning, collapsed:bool, msg:impl Message) {
        self.entries.borrow_mut().push(Event::group_begin(level,msg))
    }

    fn group_end(&self, level:level::Warning) {
        self.entries.borrow_mut().push(Event::group_end())
    }
}

impl<Level:From<level::Error>> LoggerOps<level::Error> for Logger<level::from::Warning,Level> {
    fn log(&self, level:level::Error, msg:impl Message) {
        self.entries.borrow_mut().push(Event::entry(level,msg))
    }

    fn group_begin(&self, level:level::Error, collapsed:bool, msg:impl Message) {
        self.entries.borrow_mut().push(Event::group_begin(level,msg))
    }

    fn group_end(&self, level:level::Error) {
        self.entries.borrow_mut().push(Event::group_end())
    }
}

impl<Level:From<L>,L> LoggerOps<L> for Logger<level::from::Warning,Level> {
    default fn log         (&self, _level:L, _msg:impl Message) {}
    default fn group_begin (&self, _level:L, _collapsed:bool, _msg:impl Message) {}
    default fn group_end   (&self, _level:L) {}
}




// ===================
// === Conversions ===
// ===================

impls!{ From + &From <crate::disabled::Logger> for Logger { |logger| Self::new(logger.path()) }}
