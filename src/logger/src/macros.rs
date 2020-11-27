//! Utilities for easy logger usage. Defines such macros as `debug!` or `warning!`.



// ==============
// === Macros ===
// ==============

/// Internal utility for logging macros.
#[macro_export]
macro_rules! log_template {
    ($level:path, $logger:expr, $msg:ident) => {
        $crate::LoggerOps::<$level>::log(&$logger,$level,$msg)
    };

    ($level:path, $logger:expr, || $msg:expr) => {
        $crate::LoggerOps::<$level>::log(&$logger,$level,|| $msg)
    };

    ($level:path, $logger:expr, $msg:tt) => {
        $crate::LoggerOps::<$level>::log(&$logger,$level,iformat!($msg))
    };

    ($level:path, $logger:expr, $msg:tt, || $($body:tt)*) => {
        {
            // FIXME: hardcoded false
            $crate::LoggerOps::<$level>::group_begin(&$logger,$level,false,iformat!($msg));
            let out = $($body)*;
            $crate::LoggerOps::<$level>::group_end(&$logger,$level);
            out
        }
    };
}


// === Macro Generation ===

/// Internal utility for logging macros. Generates all the logging macros.
///
/// Please note that the special pattern `$d` expands to just `$` in the generated macro from this
/// macro.
macro_rules! define_log_macros {
    ($($d:tt $name:ident $tp_name:ident;)*) => {$(
        /// $tp_name logging macro.
        #[macro_export]
        macro_rules! $name {
            ($d($d ts:tt)*) => {
                $crate::log_template!{$crate::entry::level::$tp_name,$d($d ts)*}
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
