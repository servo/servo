pub use std::*;

mod table;
mod shim;
mod alloc;
pub mod hash_map;
pub mod hash_set;

pub mod fake;

trait Recover<Q: ?Sized> {
    type Key;

    fn get(&self, key: &Q) -> Option<&Self::Key>;
    fn take(&mut self, key: &Q) -> Option<Self::Key>;
    fn replace(&mut self, key: Self::Key) -> Option<Self::Key>;
}
