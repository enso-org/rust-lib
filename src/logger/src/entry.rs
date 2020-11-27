//! Logger entry. Entry can contain message, grouping, time information, etc.

pub mod message;
pub mod level;

pub use level::DefaultLevels;
pub use level::DefaultFilter;
pub use level::filter_from;

use crate::prelude::*;

use message::Message;



// =============
// === Entry ===
// =============

/// Logger entry. Contains the message, log level, and may contain other information in the future,
/// like time, frame number, etc.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Entry<Level> {
    pub path    : ImString,
    pub level   : Level,
    pub content : Content,
}

/// Content of the entry. Can either contain simple message, or grouping information.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Content {
    Message    (String),
    GroupBegin (GroupBegin),
    GroupEnd
}

// `Content::GroupBegin` representation.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct GroupBegin {
    pub collapsed : bool,
    pub message   : String,
}

impl Content {
    /// Constructor.
    pub fn group_begin(collapsed:bool, message:String) -> Self {
        Self::GroupBegin(GroupBegin{collapsed,message})
    }

    /// Message getter. Returns `None` if it was group end.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Message(msg)  => Some(msg),
            Self::GroupBegin(t) => Some(&t.message),
            Self::GroupEnd      => None,
        }
    }
}

impl<Level> Entry<Level> {
    /// Constructor.
    pub fn message(path:ImString, level:impl Into<Level>, message:impl Message) -> Self {
        let level   = level.into();
        let content = Content::Message(message.get());
        Self {path,level,content}
    }

    /// Constructor.
    // FIXME: Unused collapsed
    pub fn group_begin
    (path:ImString, level:impl Into<Level>, message:impl Message, collapsed:bool) -> Self {
        let level   = level.into();
        let content = Content::group_begin(collapsed,message.get());
        Self {path,level,content}
    }

    /// Constructor.
    pub fn group_end(path:ImString, level:impl Into<Level>) -> Self {
        let level   = level.into();
        let content = Content::GroupEnd;
        Self {path,level,content}
    }
}
