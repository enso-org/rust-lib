//! JavaScript console consumer implementation.

use web_sys::console;

use crate::entry::Entry;
use crate::entry;
use crate::sink::consumer::Consumer;



// ==========================
// === JsConsole Consumer ===
// ==========================

/// A simple consumer which uses JavaScript console API to print hierarchical logs in a browser.
#[derive(Clone,Copy,Debug,Default)]
pub struct JsConsole;

impl<Levels> Consumer<Levels,js_sys::Array> for JsConsole {
    fn consume(&mut self, event:Entry<Levels>, message:Option<js_sys::Array>) {
        match event.content {
            entry::Content::Message(_) => {
                if let Some(msg) = message {
                    console::log(&msg);
                }
            },
            entry::Content::GroupBegin(group) => {
                if let Some(msg) = message {
                    if group.collapsed { console::group_collapsed(&msg) }
                    else               { console::group(&msg) }
                }
            },
            entry::Content::GroupEnd => {
                console::group_end()
            }
        }
    }
}
