// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod alloc;
pub mod hash_map;
pub mod hash_set;
mod shim;
mod table;

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
    /// The size we are allocating, if needed.
    allocation_size: Option<usize>,
}

impl FailedAllocationError {
    #[inline]
    pub fn new(reason: &'static str) -> Self {
        Self { reason, allocation_size: None }
    }
}

impl error::Error for FailedAllocationError {
    fn description(&self) -> &str {
        self.reason
    }
}

impl fmt::Display for FailedAllocationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.allocation_size {
            Some(size) => write!(f, "{}, allocation size: {}", self.reason, size),
            None => self.reason.fmt(f),
        }
    }
}
