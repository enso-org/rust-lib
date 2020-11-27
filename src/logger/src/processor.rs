//! Logger processor implementation.

pub mod consumer;
pub mod formatter;

use crate::prelude::*;
use crate::entry::Entry;



// ========================
// === DefaultProcessor ===
// ========================

/// Default processor implementation.
pub type DefaultProcessor =
    Pipe <
        FormatterProcessor<formatter::JsConsole>,
        ConsumerProcessor<consumer::JsConsole>
    >;



// =================
// === Processor ===
// =================

/// Trait allowing submitting entries to the processor for a particular verbosity level group
/// definition.
#[allow(missing_docs)]
pub trait Processor<Input> {
    type Output;
    fn submit(&mut self, input:Input) -> Self::Output;
}



// ==================================
// === Processors Implementations ===
// ==================================

// === Pipe ===

/// A pipe processor builder. It allows defining connected processors in a linear fashion. The macro
/// below generates a special type `Pipe` which can accept two or more processors to be connected
/// together. Because it uses default arguments, you are allowed to use it like `Pipe<P1,P2>`,
/// or `Pipe<P1,P2,P3,P4>`.
#[derive(Debug,Default)]
#[allow(missing_docs)]
pub struct PipeBuilder<First,Second> {
    pub first  : First,
    pub second : Second,
}

impl<Input,First,Second> Processor<Input> for PipeBuilder<First,Second>
where First:Processor<Input>, Second:Processor<First::Output> {
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


// === Identity Processor ===

/// Identity processor. It passes its input to output without performing any modification.
#[derive(Clone,Copy,Debug,Default)]
pub struct Identity;

impl<Input> Processor<Input> for Identity {
    type Output = Input;
    #[inline(always)]
    fn submit(&mut self, input:Input) -> Self::Output {
        input
    }
}


// === FormatterProcessor ===

/// Formatter processor. It uses the provided formatter to format its input.
#[derive(Debug,Default)]
pub struct FormatterProcessor<Formatter> {
    formatter : Formatter,
}

impl<Fmt,Lvl> Processor<Entry<Lvl>> for FormatterProcessor<Fmt>
where Fmt:formatter::Formatter<Lvl> {
    type Output = (Entry<Lvl>,Option<Fmt::Output>);
    #[inline(always)]
    fn submit(&mut self, entry:Entry<Lvl>) -> Self::Output {
        let out = <Fmt>::format(&entry.path,&entry.content);
        (entry,out)
    }
}


// === ConsumerProcessor ===

/// Consumer processor. It uses the provided consumer to consume the results, and probably print
/// them on the screen or write to a file.
#[derive(Debug,Default)]
pub struct ConsumerProcessor<Consumer> {
    consumer : Consumer,
}

impl<C,Levels,Message> Processor<(Entry<Levels>,Option<Message>)> for ConsumerProcessor<C>
where C:consumer::Consumer<Levels,Message> {
    type Output = ();
    #[inline(always)]
    fn submit(&mut self, (entry,message):(Entry<Levels>,Option<Message>)) -> Self::Output {
        self.consumer.consume(entry,message)
    }
}
