//! Logger sink implementation.

pub mod consumer;
pub mod formatter;

use crate::prelude::*;
use crate::entry::Entry;



// ===================
// === DefaultSink ===
// ===================

/// Default sink implementation.
pub type DefaultSink = Sink;



// ============
// === Sink ===
// ============

/// A sink is a combination of a formatter and consumer. The messages that enter the sink are first
/// formatted and then passed to the consumer.
#[derive(Debug,Derivative)]
#[derivative(Default(bound="Consumer:Default"))]
#[allow(missing_docs)]
pub struct Sink<Consumer=consumer::Default,Formatter=formatter::Default> {
    pub formatter : PhantomData<Formatter>,
    pub consumer  : Consumer,
}

/// Trait allowing submitting entries to the sink for a particular verbosity lever group definition.
/// This trait is implemented automatically by the `define_levels_group` macro.
#[allow(missing_docs)]
pub trait LevelSink<Level> {
    fn submit(&mut self, path:&str, entry:Entry<Level>);
}
