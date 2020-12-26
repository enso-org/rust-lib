//! Library of general data structures.

#![feature(associated_type_bounds)]
#![feature(trait_alias)]
#![feature(test)]

#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod hash_map_tree;
pub mod index;
pub mod interval_tree;
pub mod interval_tree2;
pub mod opt_vec;
pub mod text;

pub use enso_prelude as prelude;
