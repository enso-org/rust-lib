//! Built-in verbosity level definitions. Please note that the verbosity level mechanism is
//! completely user-extensible and this implementation can be user-redefined.



// ==============
// === Levels ===
// ==============

macro_rules! define_levels {
    ($($name:ident),*) => {
        #[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
        pub enum Levels {
            $($name),*
        }

        $(
            #[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Hash)]
            pub struct $name;

            impl From<$name> for Levels {
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
