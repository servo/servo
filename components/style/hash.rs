/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Reexports of hashglobe types in Gecko mode, and stdlib hashmap shims in Servo mode
//!
//! Can go away when the stdlib gets fallible collections
//! https://github.com/rust-lang/rfcs/pull/2116

use fnv;

#[cfg(feature = "gecko")]
pub use hashglobe::hash_map::HashMap;
#[cfg(feature = "gecko")]
pub use hashglobe::hash_set::HashSet;
#[cfg(feature = "gecko")]
pub use hashglobe::diagnostic::DiagnosticHashMap;

#[cfg(feature = "servo")]
pub use hashglobe::fake::{HashMap, HashSet};

/// Alias to use regular HashMaps everywhere in Servo.
#[cfg(feature = "servo")]
pub type DiagnosticHashMap<K, V, S> = HashMap<K, V, S>;

/// Appropriate reexports of hash_map types
pub mod map {
    #[cfg(feature = "gecko")]
    pub use hashglobe::hash_map::{Entry, Iter};
    #[cfg(feature = "servo")]
    pub use std::collections::hash_map::{Entry, Iter};
}

/// Hash map that uses the FNV hasher
pub type FnvHashMap<K, V> = HashMap<K, V, fnv::FnvBuildHasher>;
/// Hash set that uses the FNV hasher
pub type FnvHashSet<T> = HashSet<T, fnv::FnvBuildHasher>;
