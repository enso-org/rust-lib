//! Logger sink implementation.

pub mod consumer;
pub mod formatter;

use crate::prelude::*;
use crate::entry::Entry;



// ===================
// === DefaultSink ===
// ===================

/// Default sink implementation.
pub type DefaultSink =
    Pipe <
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



// ============================
// === Sink Implementations ===
// ============================

// === Pipe ===

/// A pipe sink builder. It allows defining connected sinks in a linear fashion. The macro below
/// generates a special type `Pipe` which can accept two or more sinks to be connected together.
/// Because it uses default arguments, you are allowed to use it like `Pipe<Sink2,Sink2>`, or
/// `Pipe<Sink1,Sink2,Sink3,Sink4>`.
#[derive(Debug,Default)]
#[allow(missing_docs)]
pub struct PipeBuilder<First,Second> {
    pub first  : First,
    pub second : Second,
}

impl<Input,First,Second> Sink<Input> for PipeBuilder<First,Second>
where First:Sink<Input>, Second:Sink<First::Output> {
    type Output = Second::Output;
    #[inline(always)]
    fn submit(&mut self, input:Input) -> Self::Output {
        self.second.submit(self.first.submit(input))
    }
}


// === Nested Pipes ===

macro_rules! define_pipes {
    ($arg:tt,$($args:tt),*) => {
        define_sub_pipes!{$arg,$($args),*}
        /// A generic pipe implementation. See docs of `PipeBuilder` to learn more.
        pub type Pipe<T=Identity,$($args=Identity),*> = $arg<T,$($args),*>;
    };
}

macro_rules! define_sub_pipes {
    () => {};
    ($arg:tt) => {};
    ($arg:tt, $($args:tt),*) => {
        /// Nested pipe. See docs of `PipeBuilder` to learn more.
        pub type $arg<$arg,$($args),*> = define_pipe_type!{$arg,$($args),*};
        define_sub_pipes! {$($args),*}
    };
}

macro_rules! define_pipe_type {
    ($arg1:tt, $arg2:tt) => {
        PipeBuilder<$arg1,$arg2>
    };
    ($arg:tt $(,$args:tt)*) => {
        PipeBuilder<$arg,define_pipe_type!{$($args),*}>
    };
}

define_pipes!(Pipe5,Pipe4,Pipe3,Pipe2,Pipe1);


// === IdentitySink ===

/// Identity sink. It passes its input to output without performing any modification.
#[derive(Clone,Copy,Debug,Default)]
pub struct Identity;

impl<Input> Sink<Input> for Identity {
    type Output = Input;
    #[inline(always)]
    fn submit(&mut self, input:Input) -> Self::Output {
        input
    }
}


// === FormatterSink ===

/// Formatter sink. It uses the provided formatter to format its input.
#[derive(Debug,Default)]
pub struct FormatterSink<Formatter> {
    formatter : Formatter,
}

impl<Fmt,Lvl> Sink<Entry<Lvl>> for FormatterSink<Fmt>
where Fmt:formatter::Formatter<Lvl> {
    type Output = (Entry<Lvl>,Option<Fmt::Output>);
    #[inline(always)]
    fn submit(&mut self, entry:Entry<Lvl>) -> Self::Output {
        let out = <Fmt>::format(&entry.path,&entry.content);
        (entry,out)
    }
}


// === ConsumerSink ===

/// Consumer sink. It uses the provided consumer to consume the results, and probably print them
/// on the screen or write to a file.
#[derive(Debug,Default)]
pub struct ConsumerSink<Consumer> {
    consumer : Consumer,
}

impl<C,Levels,Message> Sink<(Entry<Levels>,Option<Message>)> for ConsumerSink<C>
where C:consumer::Consumer<Levels,Message> {
    type Output = ();
    #[inline(always)]
    fn submit(&mut self, (entry,message):(Entry<Levels>,Option<Message>)) -> Self::Output {
        self.consumer.consume(entry,message)
    }
}
