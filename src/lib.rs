#![feature(generic_param_attrs)]
#![feature(dropck_eyepatch)]
#![feature(allocator_api)]
#![feature(alloc, shared, unique, fused, placement_new_protocol)]

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
