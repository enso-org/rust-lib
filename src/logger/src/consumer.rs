pub mod class;
pub mod js_console;

pub use class::Consumer;
pub use js_console::JsConsole;

pub type Default = JsConsole;
