//! Library of general data structures.

#![feature(associated_type_bounds)]
#![feature(trait_alias)]
#![feature(test)]

#![deny(unconditional_recursion)]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]

pub mod hash_map_tree;
pub mod index;
pub mod diet;
pub mod opt_vec;
pub mod text;

pub use enso_prelude as prelude;
