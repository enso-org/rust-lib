use crate::*;

use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
pub mod internal {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern {
        #[wasm_bindgen(js_namespace = console)]
        fn error(msg: String);

        type Error;

        #[wasm_bindgen(constructor)]
        fn new() -> Error;

        #[wasm_bindgen(structural, method, getter)]
        fn stack(error: &Error) -> String;
    }

    /// Print the current backtrace.
    pub fn backtrace() -> String {
        Error::new().stack()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    use crate::*;

    extern crate backtrace as bt;

    use bt::Backtrace;

    /// Print the current backtrace.
    pub fn backtrace() -> String {
        let bt = Backtrace::new();
        iformat!("{bt:?}")
    }
}

pub use internal::backtrace;



// ===================
// === TraceCopies ===
// ===================

/// An utility for tracing all copies of CloneRef-able entity.
///
/// This structure should be added as a field to structure implementing Clone or CloneRef. It will
/// mark each copy with unique id (the original copy has id of zeros). Once enabled, it will print
/// backtrace of each clone, clone_ref or drop operation with assigned name (the same for all
/// copies) and copy id.
#[derive(Debug,Default)]
pub struct TraceCopies {
    clone_id : Uuid,
    handle   : Rc<RefCell<Option<String>>>,
}

impl TraceCopies {
    /// Create enabled structure with appointed entity name (shared between all copies).
    pub fn enabled(name:String) -> Self {
        Self {
            clone_id : default(),
            handle   : Rc::new(RefCell::new(Some(name))),
        }
    }

    /// Assign a name to the entity (shared between all copies) and start printing logs.
    pub fn enable(&self, name:String) {
        *self.handle.borrow_mut() = Some(name);
    }
}

impl Clone for TraceCopies {
    fn clone(&self) -> Self {
        let borrow   = self.handle.borrow();
        let clone_id = Uuid::new_v4();
        let handle   = self.handle.clone();
        if let Some(name) = &*borrow {
            let bt = backtrace();
            iprintln!("Cloning {name}:{self.clone_id} -> {clone_id} {bt}");
        }
        Self {clone_id,handle}
    }
}

impl CloneRef for TraceCopies {
    fn clone_ref(&self) -> Self {
        let borrow   = self.handle.borrow();
        let clone_id = Uuid::new_v4();
        let handle   = self.handle.clone_ref();
        if let Some(name) = &*borrow {
            let bt = backtrace();
            iprintln!("Cloning-ref {name}:{self.clone_id} -> {clone_id} {bt}");
        }
        Self {clone_id,handle}
    }
}

impl Drop for TraceCopies {
    fn drop(&mut self) {
        let borrow = self.handle.borrow();
        if let Some(name) = &*borrow {
            let bt        = backtrace();
            let instances = Rc::strong_count(&self.handle) - 1;
            iprintln!("Dropping {name}:{self.clone_id} leaving {instances} instances {bt}");
        }
    }
}
