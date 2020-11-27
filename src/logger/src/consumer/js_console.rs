use web_sys::console;

use crate::entry::Entry;
use crate::entry::EntryContent;
use crate::consumer::Consumer;



// ==========================
// === JsConsole Consumer ===
// ==========================

#[derive(Debug,Default)]
pub struct JsConsole;

impl<Levels> Consumer<Levels,js_sys::Array> for JsConsole {
    fn consume(&mut self, event:Entry<Levels>, message:Option<js_sys::Array>) {
        match event.content {
            EntryContent::Message(_) => {
                if let Some(msg) = message {
                    console::log(&msg);
                }
            },
            EntryContent::GroupBegin(_) => {
                if let Some(msg) = message {
                    console::group_collapsed(&msg);
                }
            },
            EntryContent::GroupEnd => {
                console::group_end()
            }
        }
    }
}
