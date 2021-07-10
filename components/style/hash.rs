/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Reexports of hashglobe types in Gecko mode, and stdlib hashmap shims in Servo mode
//!
//! Can go away when the stdlib gets fallible collections
//! https://github.com/rust-lang/rfcs/pull/2116

use fxhash;

#[cfg(feature = "gecko")]
pub use hashglobe::hash_map::HashMap;
#[cfg(feature = "gecko")]
pub use hashglobe::hash_set::HashSet;

#[cfg(feature = "servo")]
pub use hashglobe::fake::{HashMap, HashSet};

/// Appropriate reexports of hash_map types
pub mod map {
    #[cfg(feature = "gecko")]
    pub use hashglobe::hash_map::{Entry, Iter};
    #[cfg(feature = "servo")]
    pub use std::collections::hash_map::{Entry, Iter};
}

/// Hash map that uses the Fx hasher
pub type FxHashMap<K, V> = HashMap<K, V, fxhash::FxBuildHasher>;
/// Hash set that uses the Fx hasher
pub type FxHashSet<T> = HashSet<T, fxhash::FxBuildHasher>;
