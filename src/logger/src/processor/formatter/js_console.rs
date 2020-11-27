//! JavaScript console formatter implementation.

use crate::prelude::*;

use crate::entry::level;
use crate::entry;
use crate::processor::formatter::Formatter;
use crate::processor::formatter::FormatterOutput;



// =================
// === JsConsole ===
// =================

/// A nicely looking, colorful, basic formatter for a JavaScript console.
#[derive(Clone,Copy,Debug,Default)]
pub struct JsConsole;

impl FormatterOutput for JsConsole {
    type Output = js_sys::Array;
}

impl JsConsole {
    fn format_color(path:&str, color:Option<&str>, msg:String) -> js_sys::Array {
        let msg  = format!("%c {} %c {}",path,msg).into();
        let css1 = "background:dimgray;border-radius:4px".into();
        let css2 = color.map(|c|iformat!("color:{c}")).unwrap_or_default().into();
        let arr  = js_sys::Array::new();
        arr.push(&msg);
        arr.push(&css1);
        arr.push(&css2);
        arr
    }
}


// === Impls ===

impl<Level> Formatter<Level> for JsConsole {
    default fn format(path:&str, entry:&entry::Content) -> Option<Self::Output> {
        entry.message().map(|msg| Self::format_color(path,None, msg.to_owned()))
    }
}

impl Formatter<level::Warning> for JsConsole {
    fn format(path:&str, entry:&entry::Content) -> Option<Self::Output> {
        entry.message().map(|msg| Self::format_color(path,Some("orange"),format!("[W] {}",msg)))
    }
}

impl Formatter<level::Error> for JsConsole {
    fn format(path:&str, entry:&entry::Content) -> Option<Self::Output> {
        entry.message().map(|msg| Self::format_color(path,Some("orangered"),format!("[E] {}",msg)))
    }
}
