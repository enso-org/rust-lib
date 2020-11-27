use crate::entry::Entry;



// ================
// === Consumer ===
// ================

pub trait Consumer<Levels,Message> {
    fn consume(&mut self, event:Entry<Levels>, message:Option<Message>);
}
