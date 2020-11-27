//! Logger entry. Entry can contain message, grouping, time information, etc.

pub mod message;
pub mod level;

pub use level::DefaultLevels;
pub use level::DefaultFilter;
pub use level::filter_from;

use message::Message;



// =============
// === Entry ===
// =============

/// Logger entry. Contains the message, log level, and may contain other information in the future,
/// like time, frame number, etc.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Entry<Level> {
    pub level   : Level,
    pub content : Content,
}

/// Content of the entry. Can either contain simple message, or grouping information.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Content {
    Message    (String),
    GroupBegin (String),
    GroupEnd
}

impl Content {
    /// Message getter. Returns `None` if it was group end.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Message(entry)    => Some(entry),
            Self::GroupBegin(entry) => Some(entry),
            Self::GroupEnd          => None,
        }
    }
}

impl<Level> Entry<Level> {
    /// Constructor.
    pub fn message(level:impl Into<Level>, message:impl Message) -> Self {
        let level   = level.into();
        let content = Content::Message(message.get());
        Self {level,content}
    }

    /// Constructor.
    // FIXME: Unused collapsed
    pub fn group_begin(level:impl Into<Level>, message:impl Message, _collapsed:bool) -> Self {
        let level = level.into();
        let content    = Content::GroupBegin(message.get());
        Self {level,content}
    }

    /// Constructor.
    pub fn group_end(level:impl Into<Level>) -> Self {
        let level = level.into();
        let content    = Content::GroupEnd;
        Self {level,content}
    }
}
