//! Log consumer implementation.

pub mod js_console;

pub use js_console::JsConsole;

use crate::entry::Entry;



// ========================
// === Default Consumer ===
// ========================

/// Default consumer.
pub type Default = JsConsole;


// ================
// === Consumer ===
// ================

/// Consumer takes the incoming entry and a message formatted by the used formatter and executes an
/// action, like writing the things to the console, sending them via network, or buffering in a
/// queue.
#[allow(missing_docs)]
pub trait Consumer<Levels,Message> {
    fn consume(&mut self, entry:Entry<Levels>, message:Option<Message>);
}
