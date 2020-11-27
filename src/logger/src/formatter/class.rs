use crate::entry::EntryContent;

// =================
// === Formatter ===
// =================

/// Output of a formatter as a dependent type of the formatter type. Each formatter defines its
/// output type. For example, formatters highly tailored for JavaScript console may output a special
/// console formatting values.
pub trait FormatterOutput {
    type Output;
}

/// A formatter allows formatting the incoming entry according to specific rules. Not all entries
/// need to be formatted. For example, some loggers might want to display a visual indicator when
/// a group is closed, while others will use API for that.
pub trait Formatter<Level> : FormatterOutput {
    fn format(path:&str, entry:&EntryContent) -> Option<Self::Output>;
}

/// Alias to `Formatter::format` allowing providing the type parameters on call side.
pub fn format<Fmt,Level>(path:&str, entry:&EntryContent) -> Option<Fmt::Output>
    where Fmt:Formatter<Level> {
    <Fmt>::format(path,entry)
}
