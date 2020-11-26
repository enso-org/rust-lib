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

pub use enabled::AnyLogger;

use enso_prelude::*;



// ===============
// === Message ===
// ===============

/// Message that can be logged. This trait allow a wide range of input arguments and also, allows
/// the messages to be constructed lazily, from functions.
pub trait Message              { fn get(self) -> String; }
impl      Message for &str     { fn get(self) -> String { self.into() } }
impl      Message for &&str    { fn get(self) -> String { (*self).into() } }
impl      Message for String   { fn get(self) -> String { self } }
impl      Message for &String  { fn get(self) -> String { self.clone() } }
impl      Message for &&String { fn get(self) -> String { (*self).clone() } }
impl<F,S> Message for F
where F:Fn()->S, S:Message {
    fn get(self) -> String {
        self().get()
    }
}



// ==============
// === Levels ===
// ==============

pub mod level {
    macro_rules! define_levels {
        ($($name:ident),*) => {
            #[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
            pub enum Level {
                $($name),*
            }

            $(
                #[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Hash)]
                pub struct $name;

                impl From<$name> for Level {
                    fn from(_:$name) -> Self {
                        Self::$name
                    }
                }
            )*

            pub mod from {
                $(
                    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Hash)]
                    pub struct $name;
                )*
            }
        };
    }
    define_levels!(Trace,Debug,Info,Warning,Error);
}


// =================
// === AnyLogger ===
// =================

pub trait Log<Level> {
    fn log(&self, level:Level, msg:impl Message);
}

pub trait Group<Level> {
    fn group_begin(&self, level:Level, collapsed:bool, msg:impl Message);
    fn group_end(&self, level:Level);
}




impl<T:Log<Level>,Level> Log<Level> for &T {
    fn log(&self, level:Level, msg:impl Message) {
        Log::log(*self,level,msg)
    }
}

impl<T:Group<Level>,Level> Group<Level> for &T {
    fn group_begin(&self, level:Level, collapsed:bool, msg:impl Message) {
        Group::group_begin(*self,level,collapsed,msg)
    }

    fn group_end(&self, level:Level) {
        Group::group_end(*self,level)
    }
}



// ==============
// === Macros ===
// ==============

#[macro_export]
macro_rules! log_template {
    ($level:path, $logger:expr, $msg:ident) => {
        $crate::Log::<$level>::log(&$logger,$level,$msg)
    };

    ($level:path, $logger:expr, || $msg:expr) => {
        $crate::Log::<$level>::log(&$logger,$level,|| $msg)
    };

    ($level:path, $logger:expr, $msg:tt) => {
        $crate::Log::<$level>::log(&$logger,$level,iformat!($msg))
    };

    ($level:path, $logger:expr, $msg:tt, || $($body:tt)*) => {
        {
            // FIXME: hardcoded false
            $crate::Group::<$level>::group_begin(&$logger,$level,false,iformat!($msg));
            let out = $($body)*;
            $crate::Group::<$level>::group_end(&$logger,$level);
            out
        }
    };
}


// === Macro Generation ===

macro_rules! define_log_macros {
    ($($d:tt $name:ident $tp_name:ident;)*) => {$(
        #[macro_export]
        macro_rules! $name {
            ($d($d ts:tt)*) => {
                $crate::log_template!{$crate::level::$tp_name,$d($d ts)*}
            };
        }
    )*};
}

define_log_macros!{
    $ trace   Trace;
    $ debug   Debug;
    $ info    Info;
    $ warning Warning;
    $ error   Error;
}
