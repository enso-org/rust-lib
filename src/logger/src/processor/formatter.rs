//! Log formatter implementation.

pub mod js_console;

pub use js_console::JsConsole;

use crate::entry;



// =========================
// === Default Formatter ===
// =========================

/// Default log formatter.
pub type Default = JsConsole;



// =================
// === Formatter ===
// =================

/// Output of a formatter as a dependent type of the formatter type. Each formatter defines its
/// output type. For example, formatters highly tailored for JavaScript console may output a special
/// console formatting values.
#[allow(missing_docs)]
pub trait FormatterOutput {
    type Output;
}

/// A formatter allows formatting the incoming entry according to specific rules. Not all entries
/// need to be formatted. For example, some loggers might want to display a visual indicator when
/// a group is closed, while others will use API for that.
#[allow(missing_docs)]
pub trait Formatter<Level> : FormatterOutput {
    fn format(path:&str, entry:&entry::Content) -> Option<Self::Output>;
}

/// Alias to `Formatter::format` allowing providing the type parameters on call side.
pub fn format<Fmt,Level>(path:&str, entry:&entry::Content) -> Option<Fmt::Output>
where Fmt:Formatter<Level> {
    <Fmt>::format(path,entry)
}
