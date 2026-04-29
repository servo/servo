/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of `setlike<...>` and `maplike<..., ...>` WebIDL declarations.

pub(crate) use script_bindings::like::*;

/// Usage:
/// ```rust
/// pub(crate) struct TestBindingSetlike {
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

/// Usage:
/// ```rust
/// pub(crate) struct TestBindingMaplike {
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
