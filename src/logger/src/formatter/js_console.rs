use crate::level;
use crate::formatter::Formatter;
use crate::formatter::FormatterOutput;
use crate::entry::EntryContent;



// =================
// === JsConsole ===
// =================

/// A nicely looking, colorful, basic formatter for JavaScript Console.
#[derive(Debug,Default)]
pub struct JsConsole;

impl FormatterOutput for JsConsole {
    type Output = js_sys::Array;
}

impl JsConsole {
    fn format_color(path:&str, color:&str, msg:String) -> js_sys::Array {
        let msg  = format!("%c {} %c {}",path,msg).into();
        let css1 = "background:dimgray;border-radius:4px".into();
        let css2 = format!("color:{}",color).into();
        let arr  = js_sys::Array::new();
        arr.push(&msg);
        arr.push(&css1);
        arr.push(&css2);
        arr
    }
}


// === Impls ===

impl Formatter<level::Debug> for JsConsole {
    fn format(path:&str, event:&EntryContent) -> Option<Self::Output> {
        event.entry().map(|msg| Self::format_color(path, "red", msg.to_owned()))
    }
}

impl Formatter<level::Warning> for JsConsole {
    fn format(path:&str, event:&EntryContent) -> Option<Self::Output> {
        event.entry().map(|mmsg| Self::format_color(path, "orange", format!("[W] {}",mmsg)))
    }
}
