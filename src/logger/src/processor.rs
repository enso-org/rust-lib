//! Logger processor implementation.

pub mod consumer;
pub mod formatter;

use crate::prelude::*;
use crate::entry::Entry;
use crate::entry::level::DefaultLevels;
use wasm_bindgen::prelude::*;



// ===========================
// === JavaScript Bindings ===
// ===========================

mod js {
    use super::*;
    #[wasm_bindgen(inline_js = "
        export function setup_logs_flush(fn) {
            let oldShowLogs = window.showLogs
            window.showLogs = () => {
                if (oldShowLogs) { oldShowLogs() }
                fn()
            }
        }

        export function check_auto_flush() {
            return (console.autoFlush === true)
        }
    ")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn setup_logs_flush(closure:&Closure<dyn Fn()>);

        /// When the `showLogs` function is evaluated, the `autoFlush` flag is set to true. This
        /// may happen even before the WASM file is loaded, so it's worth checking whether it
        /// happened on startup.
        #[allow(unsafe_code)]
        pub fn check_auto_flush() -> bool;
    }
}



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


// === Formatter ===

/// Formatter processor. It uses the provided formatter to format its input.
#[derive(Debug,Default)]
pub struct Formatter<T> {
    formatter : T,
}

impl<Fmt,Lvl> Processor<Entry<Lvl>> for Formatter<Fmt>
where Fmt:formatter::GenericDefinition<Lvl> {
    type Output = (Entry<Lvl>,Option<Fmt::Output>);
    #[inline(always)]
    fn submit(&mut self, entry:Entry<Lvl>) -> Self::Output {
        let out = <Fmt>::generic_format(&entry);
        (entry,out)
    }
}


// === Consumer ===

/// Consumer processor. It uses the provided consumer to consume the results, and probably print
/// them on the screen or write to a file.
#[derive(Debug,Default)]
pub struct Consumer<T> {
    consumer : T,
}

impl<C,Levels,Message> Processor<(Entry<Levels>,Option<Message>)> for Consumer<C>
where C:consumer::Definition<Levels,Message> {
    type Output = ();
    #[inline(always)]
    fn submit(&mut self, (entry,message):(Entry<Levels>,Option<Message>)) -> Self::Output {
        self.consumer.consume(entry,message)
    }
}


// === Buffer ===

#[derive(Debug,Derivative)]
#[allow(missing_docs)]
pub struct Buffer<Input,Next> {
    model   : Rc<RefCell<BufferModel<Input,Next>>>,
    closure : Closure<dyn Fn()>,
}

impl<Input,Next> Default for Buffer<Input,Next>
    where Input:'static, Next:'static+Default+Processor<Input> {
    fn default() -> Self {
        let model   = Rc::new(RefCell::new(BufferModel::<Input,Next>::default()));
        let closure = Closure::new(f!(model.borrow_mut().flush_and_enable_auto_flush()));
        js::setup_logs_flush(&closure);
        Self{model,closure}
    }
}

impl<Input,Next> Processor<Input> for Buffer<Input,Next>
    where Next:Processor<Input> {
    type Output = ();
    #[inline(always)]
    fn submit(&mut self, input:Input) {
        self.model.borrow_mut().submit(input);
    }
}

#[derive(Debug,Derivative)]
#[allow(missing_docs)]
pub struct BufferModel<Input,Next> {
    buffer     : Vec<Input>,
    auto_flush : bool,
    next       : Next,
}

impl<Input,Next:Default> Default for BufferModel<Input,Next> {
    fn default() -> Self {
        let auto_flush = js::check_auto_flush();
        let buffer     = default();
        let next       = default();
        Self {buffer,auto_flush,next}
    }
}

impl<Input,Next> BufferModel<Input,Next>
    where Next:Processor<Input> {
    /// Submit the input to the buffer or the subsequent processor in case the `auto_flush` is
    /// enabled.
    pub fn submit(&mut self, input:Input) {
        if self.auto_flush {
            self.next.submit(input);
        } else {
            self.buffer.push(input);
        }
    }

    /// Pass all buffered entries to the subsequent processor.
    pub fn flush(&mut self) {
        for input in mem::take(&mut self.buffer) {
            self.next.submit(input);
        }
    }

    /// Pass all buffered entries to the subsequent processor and set the `auto_flush` flag to on.
    pub fn flush_and_enable_auto_flush(&mut self) {
        self.flush();
        self.auto_flush = true;
    }
}


// === Global ===

#[derive(Debug,Default)]
#[allow(missing_docs)]
pub struct Global<Processor> {
    processor : PhantomData<Processor>
}

impl<P,Input> Processor<Input> for Global<P>
    where P:GlobalProcessor, P::Processor:'static+Processor<Input> {
    type Output = <<P as GlobalProcessor>::Processor as Processor<Input>>::Output;
    #[inline(always)]
    fn submit(&mut self, entry:Input) -> Self::Output {
        global_processor::<P>().submit(entry)
    }
}

/// Abstraction for global processors. Global processors may be insanely useful to optimize the
/// logging performance. You can, for example, define a single global processor and redirect all
/// loggers to it. The single global processor can have a buffer layer, which will buffer messages
/// without formatting them and will format all of them and print them to the screen on-demand only.
#[allow(missing_docs)]
pub trait GlobalProcessor {
    type Processor;
    fn get_mut() -> &'static mut Self::Processor;
}

/// Get a reference to a global processor. Read docs of `GlobalProcessor` to learn more.
pub fn global_processor<T:GlobalProcessor>() -> &'static mut T::Processor {
    T::get_mut()
}


/// Define a global processor based on the provided type. Read the docs of `GlobalProcessor` to
/// learn more.
#[macro_export]
macro_rules! define_global_processor {
    ($name:ident = $tp:ty;) => {
        /// Global processor definition.
        #[derive(Copy,Clone,Debug,Default)]
        pub struct $name;
        paste::item! {
            #[allow(non_upper_case_globals)]
            static mut [<$name _STATIC_MUT>]: Option<$tp> = None;
        }
        impl GlobalProcessor for $name {
            type Processor = $tp;
            paste::item! {
                #[allow(unsafe_code)]
                fn get_mut() -> &'static mut Self::Processor {
                    unsafe {
                        match &mut [<$name _STATIC_MUT>] {
                            Some(t) => t,
                            None    => {
                                let processor = default();
                                [<$name _STATIC_MUT>] = Some(processor);
                                [<$name _STATIC_MUT>].as_mut().unwrap()
                            }
                        }
                    }
                }
            }
        }
    };
}



// ========================
// === DefaultProcessor ===
// ========================

/// Default processor implementation.
pub type DefaultProcessor = Global<DefaultGlobalProcessor>;

define_global_processor! {
    DefaultGlobalProcessor =
        Buffer<Entry<DefaultLevels>,
            Pipe <
                Formatter<formatter::JsConsole>,
                Consumer<consumer::JsConsole>
            >
        >;
}
