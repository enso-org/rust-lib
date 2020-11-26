//! Primitive operations on a logger.

use crate::message::Message;



// =================
// === LoggerOps ===
// =================

/// Primitive operations on a logger.
#[allow(missing_docs)]
pub trait LoggerOps<Level> {
    fn log         (&self, level:Level, msg:impl Message);
    fn group_begin (&self, level:Level, collapsed:bool, msg:impl Message);
    fn group_end   (&self, level:Level);
}


// === Ref Impl ===

impl<T:LoggerOps<Level>,Level> LoggerOps<Level> for &T {
    fn log(&self, level:Level, msg:impl Message) {
        LoggerOps::log(*self,level,msg)
    }

    fn group_begin(&self, level:Level, collapsed:bool, msg:impl Message) {
        LoggerOps::group_begin(*self,level,collapsed,msg)
    }

    fn group_end(&self, level:Level) {
        LoggerOps::group_end(*self,level)
    }
}
