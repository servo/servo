#![feature(generic_param_attrs)]
#![feature(dropck_eyepatch)]
#![feature(allocator_api)]
#![feature(rand, alloc, needs_drop, shared, unique, fused, placement_new_protocol)]

extern crate alloc;
extern crate rand;


pub use std::*;

mod hash;

pub mod hash_map {
    pub use super::hash::map::*;
}

pub mod hash_set {
    pub use super::hash::set::*;
}
