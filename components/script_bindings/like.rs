/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of `setlike<...>` and `maplike<..., ...>` WebIDL declarations.

use js::conversions::ToJSValConvertible;

use crate::iterable::Iterable;

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
    fn get_value_at_index(&self, index: u32) -> <T as Maplike>::Value {
        // SAFETY: we are checking bounds manually
        self.get_index(index).unwrap().1
    }

    #[inline]
    fn get_key_at_index(&self, index: u32) -> <T as Maplike>::Key {
        // SAFETY: we are checking bounds manually
        self.get_index(index).unwrap().0
    }
}
