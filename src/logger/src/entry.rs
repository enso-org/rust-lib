
use crate::message::Message;


// =============
// === Entry ===
// =============

/// Logger entry. Contains the message, log level, and may contain other information in the future,
/// like time, frame number, etc.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Entry<Level> {
    pub level   : Level,
    pub content : EntryContent,
}

#[derive(Debug)]
pub enum EntryContent {
    Message    (String),
    GroupBegin (String),
    GroupEnd
}

impl EntryContent {
    pub fn entry(&self) -> Option<&str> {
        match self {
            Self::Message(entry)      => Some(entry),
            Self::GroupBegin(entry) => Some(entry),
            Self::GroupEnd          => None,
        }
    }
}

impl<Level> Entry<Level> {
    pub fn message(level:impl Into<Level>, message:impl Message) -> Self {
        let level = level.into();
        let content    = EntryContent::Message(message.get());
        Self {level,content}
    }

    pub fn group_begin(level:impl Into<Level>, message:impl Message, collapsed:bool) -> Self {
        let level = level.into();
        let content    = EntryContent::GroupBegin(message.get());
        Self {level,content}
    }

    pub fn group_end(level:impl Into<Level>) -> Self {
        let level = level.into();
        let content    = EntryContent::GroupEnd;
        Self {level,content}
    }

    pub fn casted<L>(self, level:L) -> Entry<L> {
        let content = self.content;
        Entry{level,content}
    }
}