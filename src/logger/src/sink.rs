pub mod consumer;
pub mod formatter;

use crate::prelude::*;
use crate::entry::message::Message;
use crate::entry::Entry;

use wasm_bindgen::JsValue;


#[derive(Debug,Derivative)]
#[derivative(Default(bound="Consumer:Default"))]
#[allow(missing_docs)]
pub struct Sink<Consumer=consumer::Default,Formatter=formatter::Default> {
    pub formatter : PhantomData<Formatter>,
    pub consumer  : Consumer,
}

impl<Fmt> Sink<Fmt> {
    fn format_color(&self, path:&str, color:&str, msg:String) -> (JsValue,JsValue,JsValue) {
        let msg  = format!("%c {} %c {}",path,msg).into();
        let css1 = "background:dimgray;border-radius:4px".into();
        let css2 = format!("color:{}",color).into();
        (msg,css1,css2)
    }

    fn format_warn(&self, path:&str, msg:impl Message) -> (JsValue,JsValue,JsValue) {
        self.format_color(path,"orange",format!("[W] {}",msg.get()))
    }
}

pub trait LevelGroupSink<Level> {
    fn submit(&mut self, path:&str, event:Entry<Level>);
}
