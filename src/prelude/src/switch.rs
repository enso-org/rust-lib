//! Type `Switch` represents a value which can be turned on or off. It is similar to `(T,bool)`,
//! but with many utility functions allowing for convenient workflow. An example use case would be
//! passing around information if a particular node in a tree was hovered or not. You can pass
//! `Switch<Crumb>` value then, where `Crumb` stores a path to the node from the root of the tree.



// ==============
// === Switch ===
// ==============

/// The `Switch` type. Read module docs to learn more.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq,Hash)]
#[allow(missing_docs)]
pub struct Switch<T> {
    pub value : T,
    is_on     : bool,
}


// === Construction ===

impl<T> Switch<T> {
    /// Constructor.
    pub fn new(value:T, is_on:bool) -> Self {
        Self {value,is_on}
    }

    /// Constructor.
    #[allow(non_snake_case)]
    pub fn On(value:T) -> Self {
        Self::new(value,true)
    }

    /// Constructor.
    #[allow(non_snake_case)]
    pub fn Off(value:T) -> Self {
        Self::new(value,false)
    }
}


// === Modifiers ===

impl<T> Switch<T> {
    /// Change the on / off status.
    pub fn switch(&mut self, is_on:bool) {
        self.is_on = is_on;
    }

    /// Toggle the on / off status.
    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
    }

    /// Change the on / off status while consuming the value.
    pub fn switched(mut self, is_on:bool) -> Self {
        self.switch(is_on);
        self
    }

    /// Toggle the on / off status while consuming the value.
    pub fn toggled(mut self) -> Self {
        self.toggle();
        self
    }
}


// === Status ===

impl<T> Switch<T> {
    /// Check whether the value is enabled.
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    /// Check whether the value is disabled.
    pub fn is_off(&self) -> bool {
        !self.is_on()
    }
}


// === Getters ===

impl<T> Switch<T> {
    /// Get the value if it was turned on.
    pub fn on(&self) -> Option<&T> {
        if self.is_on() { Some(&self.value) } else { None }
    }

    /// Get the value if it was turned off.
    pub fn off(&self) -> Option<&T> {
        if self.is_off() { Some(&self.value) } else { None }
    }

    /// Get the value if it was turned on while consuming self.
    pub fn into_on(self) -> Option<T> {
        if self.is_on() { Some(self.value) } else { None }
    }

    /// Get the value if it was turned off while consuming self.
    pub fn into_off(self) -> Option<T> {
        if self.is_off() { Some(self.value) } else { None }
    }
}
