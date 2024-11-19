/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Implementation of `setlike<...>` and `maplike<..., ...>` WebIDL declarations.

use std::cmp::Eq;
use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};
use js::conversions::ToJSValConvertible;

use super::iterable::Iterable;

/// Every Setlike dom_struct must implement this to provide access to underlying storage
/// so codegen can automatically generate all setlike methods
///
/// In case you use a type that implements Setlike as underlying storage it's recommended to use `setlike` macro.
// In webidl: `setlike<Key>`
pub trait Setlike {
    /// The type of the key of the set.
    type Key: ToJSValConvertible + Clone; // clone is for impl<T: Setlike> Maplike for T

    fn get_index(&self, index: u32) -> Option<Self::Key>;

    fn size(&self) -> u32;
    fn add(&self, key: Self::Key);
    fn has(&self, key: Self::Key) -> bool;
    fn clear(&self);
    fn delete(&self, key: Self::Key) -> bool;
}

// we can only have one iterable for T
// so we have impl<T: Maplike> Iterable for T
// and minimal:
impl<T: Setlike> Maplike for T {
    type Key = <T as Setlike>::Key;

    type Value = <T as Setlike>::Key;

    #[inline]
    fn get_index(&self, index: u32) -> Option<(Self::Key, Self::Value)> {
        self.get_index(index).map(|k| (k.clone(), k))
    }

    fn get(&self, _key: Self::Key) -> Option<Self::Value> {
        unimplemented!()
    }

    #[inline]
    fn size(&self) -> u32 {
        self.size()
    }

    fn set(&self, _key: Self::Key, _value: Self::Value) {
        unimplemented!()
    }

    fn has(&self, _key: Self::Key) -> bool {
        unimplemented!()
    }

    fn clear(&self) {
        unimplemented!()
    }

    fn delete(&self, _key: Self::Key) -> bool {
        unimplemented!()
    }
}

/// Usage:
/// ```rust
/// pub struct TestBindingSetlike {
///     // setlike<DOMString>
///     internal: DomRefCell<IndexSet<DOMString>>,
/// }
/// impl Setlike for TestBindingSetlike {
///     type Key = DOMString;
///     setlike!(self, internal);
/// }
/// ```
#[macro_export]
macro_rules! setlike {
    ( $self:ident, $x:ident ) => {
        #[inline(always)]
        fn get_index(&$self, index: u32) -> Option<Self::Key> {
            $self.$x.get_index(index)
        }

        #[inline(always)]
        fn size(&$self) -> u32 {
            $self.$x.size()
        }

        #[inline(always)]
        fn add(&$self, key: Self::Key) {
            $self.$x.add(key)
        }

        #[inline(always)]
        fn has(&$self, key: Self::Key) -> bool {
            $self.$x.has(key)
        }

        #[inline(always)]
        fn clear(&$self) {
            $self.$x.clear()
        }

        #[inline(always)]
        fn delete(&$self, key: Self::Key) -> bool {
            $self.$x.delete(key)
        }
    };
}

/// Every Maplike dom_struct must implement this
/// to provide access to underlying storage
/// so codegen can automatically generate all maplike methods
///
/// In case you use a type that implements Maplike as underlying storage it's recommended to use `maplike` macro.
// In webidl: `maplike<Key, Value>`
pub trait Maplike {
    /// The type of the key of the map.
    type Key: ToJSValConvertible;
    /// The type of the value of the map.
    type Value: ToJSValConvertible;

    fn get_index(&self, index: u32) -> Option<(Self::Key, Self::Value)>;

    fn get(&self, key: Self::Key) -> Option<Self::Value>;
    fn size(&self) -> u32;
    fn set(&self, key: Self::Key, value: Self::Value);
    fn has(&self, key: Self::Key) -> bool;
    fn clear(&self);
    fn delete(&self, key: Self::Key) -> bool;
}

impl<T: Maplike> Iterable for T {
    type Key = T::Key;

    type Value = T::Value;

    #[inline]
    fn get_iterable_length(&self) -> u32 {
        self.size()
    }

    #[inline]
    fn get_value_at_index(&self, index: u32) -> Self::Value {
        // SAFETY: we are checking bounds manually
        self.get_index(index).unwrap().1
    }

    #[inline]
    fn get_key_at_index(&self, index: u32) -> Self::Key {
        // SAFETY: we are checking bounds manually
        self.get_index(index).unwrap().0
    }
}

/// Usage:
/// ```rust
/// pub struct TestBindingMaplike {
///     // maplike<DOMString, long>
///     internal: DomRefCell<IndexMap<DOMString, i32>>,
/// }
/// impl Maplike for TestBindingSetlike {
///     type Key = DOMString;
///     type Value = i32;
///     maplike!(self, internal);
/// }
/// ```
#[macro_export]
macro_rules! maplike {
    ( $self:ident, $internal:ident ) => {
        #[inline(always)]
        fn get_index(&$self, index: u32) -> Option<(Self::Key, Self::Value)> {
            $self.$internal.get_index(index)
        }

        #[inline(always)]
        fn get(&$self, key: Self::Key) -> Option<Self::Value> {
            $self.$internal.get(key)
        }

        #[inline(always)]
        fn size(&$self) -> u32 {
            $self.$internal.size()
        }

        #[inline(always)]
        fn set(&$self, key: Self::Key, value: Self::Value) {
            $self.$internal.set(key, value)
        }

        #[inline(always)]
        fn has(&$self, key: Self::Key) -> bool {
            $self.$internal.has(key)
        }

        #[inline(always)]
        fn clear(&$self) {
            $self.$internal.clear()
        }

        #[inline(always)]
        fn delete(&$self, key: Self::Key) -> bool {
            $self.$internal.delete(key)
        }
    };
}
