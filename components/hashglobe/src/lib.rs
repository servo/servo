pub use std::*;

extern crate heapsize;

mod table;
mod shim;
mod alloc;
pub mod hash_map;
pub mod hash_set;

pub mod fake;

use std::{error, fmt};

trait Recover<Q: ?Sized> {
    type Key;

    fn get(&self, key: &Q) -> Option<&Self::Key>;
    fn take(&mut self, key: &Q) -> Option<Self::Key>;
    fn replace(&mut self, key: Self::Key) -> Option<Self::Key>;
}

#[derive(Debug)]
pub struct FailedAllocationError {
    reason: &'static str,
}

impl error::Error for FailedAllocationError {
    fn description(&self) -> &str {
        self.reason
    }
}

impl fmt::Display for FailedAllocationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.reason.fmt(f)
    }
}
