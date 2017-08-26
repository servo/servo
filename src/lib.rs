
#![feature(allocator_api)]
#![feature(alloc)]

extern crate alloc;

pub use std::*;

mod bench;
mod table;
mod shim;
#[path="alloc.rs"]
mod alloc2;
pub mod hash_map;
pub mod hash_set;

trait Recover<Q: ?Sized> {
    type Key;

    fn get(&self, key: &Q) -> Option<&Self::Key>;
    fn take(&mut self, key: &Q) -> Option<Self::Key>;
    fn replace(&mut self, key: Self::Key) -> Option<Self::Key>;
}
