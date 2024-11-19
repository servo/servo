/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Implementation of `setlike<...>` and `maplike<..., ...>` WebIDL declarations.

use std::cmp::Eq;
use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};
use js::conversions::ToJSValConvertible;
pub use script_bindings::like::*;

use super::iterable::Iterable;
use crate::dom::bindings::cell::DomRefCell;

impl<K> Setlike for DomRefCell<IndexSet<K>>
where
    K: ToJSValConvertible + Eq + PartialEq + Hash + Clone,
{
    type Key = K;

    #[inline(always)]
    fn get_index(&self, index: u32) -> Option<Self::Key> {
        self.borrow().get_index(index as usize).cloned()
    }

    #[inline(always)]
    fn size(&self) -> u32 {
        self.borrow().len() as u32
    }

    #[inline(always)]
    fn add(&self, key: Self::Key) {
        self.borrow_mut().insert(key);
    }

    #[inline(always)]
    fn has(&self, key: Self::Key) -> bool {
        self.borrow().contains(&key)
    }

    #[inline(always)]
    fn clear(&self) {
        self.borrow_mut().clear()
    }

    #[inline(always)]
    fn delete(&self, key: Self::Key) -> bool {
        self.borrow_mut().shift_remove(&key)
    }
}

impl<K, V> Maplike for DomRefCell<IndexMap<K, V>>
where
    K: ToJSValConvertible + Eq + PartialEq + Hash + Clone,
    V: ToJSValConvertible + Clone,
{
    type Key = K;
    type Value = V;

    #[inline(always)]
    fn get_index(&self, index: u32) -> Option<(Self::Key, Self::Value)> {
        self.borrow()
            .get_index(index as usize)
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
    }

    #[inline(always)]
    fn get(&self, key: Self::Key) -> Option<Self::Value> {
        self.borrow().get(&key).cloned()
    }

    #[inline(always)]
    fn size(&self) -> u32 {
        self.borrow().len() as u32
    }

    #[inline(always)]
    fn set(&self, key: Self::Key, value: Self::Value) {
        self.borrow_mut().insert(key, value);
    }

    #[inline(always)]
    fn has(&self, key: Self::Key) -> bool {
        self.borrow().contains_key(&key)
    }

    #[inline(always)]
    fn clear(&self) {
        self.borrow_mut().clear()
    }

    #[inline(always)]
    fn delete(&self, key: Self::Key) -> bool {
        self.borrow_mut().shift_remove(&key).is_some()
    }
}
