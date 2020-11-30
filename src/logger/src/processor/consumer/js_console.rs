//! JavaScript console consumer implementation.

use web_sys::console;

use crate::entry::Entry;
use crate::entry;
use crate::processor::consumer;
use wasm_bindgen::prelude::*;



mod js {
    use super::*;
    #[wasm_bindgen(inline_js = "
        export function console_group_end() {
            console.groupEnd()
        }
    ")]
    extern "C" {
        /// FIXME[WD]: Issue https://github.com/rustwasm/wasm-bindgen/issues/2376
        /// This is just the same as `wasm_bindgen::console::group_end` with one important
        /// difference. It seems that `wasm_bindgen` somehow caches all functions without args on
        /// initialization, and thus, as we are redefining what `console.group_end` is in JS, the
        /// function provided by the library, unlike this one, does not reflect the change.
        #[allow(unsafe_code)]
        pub fn console_group_end();
    }
}



// ==========================
// === JsConsole Consumer ===
// ==========================

/// A simple consumer which uses JavaScript console API to print hierarchical logs in a browser.
#[derive(Clone,Copy,Debug,Default)]
pub struct JsConsole;

impl<Levels> consumer::Definition<Levels,js_sys::Array> for JsConsole {
    fn consume(&mut self, event:Entry<Levels>, message:Option<js_sys::Array>) {
        match &event.content {
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
                js::console_group_end()
            }
        }
    }
}