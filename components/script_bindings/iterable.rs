/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::conversions::ToJSValConvertible;

use crate::utils::DOMClass;

/// The values that an iterator will iterate over.
#[derive(JSTraceable, MallocSizeOf)]
pub enum IteratorType {
    /// The keys of the iterable object.
    Keys,
    /// The values of the iterable object.
    Values,
    /// The keys and values of the iterable object combined.
    Entries,
}

/// A DOM object that can be iterated over using a pair value iterator.
pub trait Iterable {
    /// The type of the key of the iterator pair.
    type Key: ToJSValConvertible;
    /// The type of the value of the iterator pair.
    type Value: ToJSValConvertible;
    /// Return the number of entries that can be iterated over.
    fn get_iterable_length(&self) -> u32;
    /// Return the value at the provided index.
    fn get_value_at_index(&self, index: u32) -> Self::Value;
    /// Return the key at the provided index.
    fn get_key_at_index(&self, index: u32) -> Self::Key;
}

/// A version of the [IDLInterface] trait that is specific to types that have
/// iterators defined for them. This allows the `script` crate to define the
/// derives check for the concrete interface type, while the [IteratableIterator]
/// type defined in this module can be parameterized over an unknown generic.
pub trait IteratorDerives {
    fn derives(class: &'static DOMClass) -> bool;
}
