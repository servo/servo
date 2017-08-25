#![feature(allocator_api)]
#![feature(alloc, shared, unique)]

extern crate alloc;


pub use std::*;

#[path="hash/mod.rs"]
mod impls;

pub mod hash_map {
    pub use super::impls::map::*;
}

pub mod hash_set {
    pub use super::impls::set::*;
}
