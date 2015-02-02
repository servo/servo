/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This file stolen wholesale from rustc/src/librustc/util/nodemap.rs

use std::default::Default;
use std::hash::{Hasher, Writer};

/// A speedy hash algorithm for node ids and def ids. The hashmap in
/// libcollections by default uses SipHash which isn't quite as speedy as we
/// want. In the compiler we're not really worried about DOS attempts, so we
/// just default to a non-cryptographic hash.
///
/// This uses FNV hashing, as described here:
/// http://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
#[allow(missing_copy_implementations)]
pub struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> FnvHasher { FnvHasher(0xcbf29ce484222325) }
}

impl Hasher for FnvHasher {
    type Output = u64;
    fn reset(&mut self) { *self = Default::default(); }
    fn finish(&self) -> u64 { self.0 }
}

impl Writer for FnvHasher {
    fn write(&mut self, bytes: &[u8]) {
        let FnvHasher(mut hash) = *self;
        for byte in bytes.iter() {
            hash = hash ^ (*byte as u64);
            hash = hash * 0x100000001b3;
        }
        *self = FnvHasher(hash);
    }
}
