//! Built-in verbosity level definitions and a set of utilities to define custom levels. Please note
//! that the verbosity level mechanism is completely user-extensible and this implementation can be
//! completely redefined by the user.

use crate::prelude::*;
use crate::processor::formatter;
use crate::entry::Entry;
use crate::entry::level;



// ==============
// === Macros ===
// ==============

/// Utility for defining verbosity levels. See an example usage below.
#[macro_export]
macro_rules! define_levels {
    ($($name:ident),*) => {
        $(
            /// Log level.
            #[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Hash)]
            pub struct $name;
        )*

        /// Allows compile-time filtering of all entries from (more important) than the selected
        /// level. For example, `filter_from::Warning` will keep warnings and errors only.
        pub mod filter_from {
            $(
                /// Filtering log level.
                #[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Hash)]
                pub struct $name;
            )*
        }
    };
}


/// Utility for defining verbosity level groups. See an example usage below.
#[macro_export]
macro_rules! define_levels_group {
    ($group_name:ident { $($name:ident),* $(,)?} ) => {
        /// Possible verbosity levels enum.
        #[allow(missing_docs)]
        #[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
        pub enum $group_name {
            $($name),*
        }

        $(
            impl From<$name> for $group_name {
                fn from(_:$name) -> Self {
                    Self::$name
                }
            }
        )*

        impl<T> formatter::GenericDefinition<DefaultLevels> for T
        where $(T : formatter::Definition<level::$name>),* {
            fn generic_format(entry:&Entry<DefaultLevels>) -> Option<Self::Output> {
                match entry.level {
                    $(
                        DefaultLevels::$name =>
                            formatter::format::<T,level::$name> (&entry.gen_entry)
                    ),*
                }
            }
        }
    };
}



// =======================
// === Built-in Levels ===
// =======================

define_levels!(Trace,Debug,Info,Warning,Error);
define_levels_group!(DefaultLevels {Trace,Debug,Info,Warning,Error});



// =====================
// === DefaultFilter ===
// =====================

/// Default compile-time logger filtering. Keeps all logs.
pub type DefaultFilter = filter_from::Trace;




