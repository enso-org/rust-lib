//! Logger sink implementation.

pub mod consumer;
pub mod formatter;

use crate::prelude::*;
use crate::entry;
use crate::entry::Entry;



// ===================
// === DefaultSink ===
// ===================

/// Default sink implementation.
pub type DefaultSink =
    Pipe<
        FormatterSink<formatter::JsConsole>,
        ConsumerSink<consumer::JsConsole>
    >;



// ============
// === Sink ===
// ============

/// Trait allowing submitting entries to the sink for a particular verbosity lever group definition.
/// This trait is implemented automatically by the `define_levels_group` macro.
#[allow(missing_docs)]
pub trait Sink<Input> {
    type Output;
    fn submit(&mut self, input:Input) -> Self::Output;
}


// =====================

#[derive(Debug,Default)]
pub struct FormatterSink<Formatter> {
    formatter : Formatter,
}

impl<Fmt,Lvl> Sink<(ImString,Entry<Lvl>)> for FormatterSink<Fmt>
where Fmt:formatter::Formatter<Lvl> {
    type Output = (Entry<Lvl>,Option<Fmt::Output>);
    fn submit(&mut self, (path,entry):(ImString,Entry<Lvl>)) -> Self::Output {
        let out = <Fmt>::format(&path,&entry.content);
        (entry,out)
    }
}


// =====================

#[derive(Debug,Default)]
pub struct ConsumerSink<Consumer> {
    consumer : Consumer,
}

impl<C,Levels,Message> Sink<(Entry<Levels>,Option<Message>)> for ConsumerSink<C>
where C:consumer::Consumer<Levels,Message> {
    type Output = ();
    fn submit(&mut self, (entry,message):(Entry<Levels>,Option<Message>)) -> Self::Output {
        self.consumer.consume(entry,message)
    }
}


// =====================

#[derive(Debug,Default)]
pub struct Pipe<First,Second> {
    pub first  : First,
    pub second : Second,
}

impl<Input,First,Second> Sink<Input> for Pipe<First,Second>
where First:Sink<Input>, Second:Sink<First::Output> {
    type Output = Second::Output;
    fn submit(&mut self, input:Input) -> Self::Output {
        self.second.submit(self.first.submit(input))
    }
}
