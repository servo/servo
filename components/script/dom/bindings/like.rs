/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of `setlike<...>` and `maplike<..., ...>` WebIDL declarations.

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
        fn get_index(&$self, cx: &mut ::js::context::JSContext, index: u32) -> Option<Self::Key> {
            $self.$x.get_index(cx, index)
        }

        #[inline(always)]
        fn size(&$self, cx: &mut ::js::context::JSContext) -> u32 {
            $self.$x.size(cx)
        }

        #[inline(always)]
        fn add(&$self, cx: &mut ::js::context::JSContext, key: Self::Key) {
            $self.$x.add(cx, key)
        }

        #[inline(always)]
        fn has(&$self, cx: &mut ::js::context::JSContext, key: Self::Key) -> bool {
            $self.$x.has(cx, key)
        }

        #[inline(always)]
        fn clear(&$self, cx: &mut ::js::context::JSContext) {
            $self.$x.clear(cx)
        }

        #[inline(always)]
        fn delete(&$self, cx: &mut ::js::context::JSContext, key: Self::Key) -> bool {
            $self.$x.delete(cx, key)
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
        fn get_index(&$self, cx: &mut ::js::context::JSContext, index: u32) -> Option<(Self::Key, Self::Value)> {
            $self.$internal.get_index(cx, index)
        }

        #[inline(always)]
        fn get(&$self, cx: &mut ::js::context::JSContext, key: Self::Key) -> Option<Self::Value> {
            $self.$internal.get(cx, key)
        }

        #[inline(always)]
        fn size(&$self, cx: &mut ::js::context::JSContext) -> u32 {
            $self.$internal.size(cx)
        }

        #[inline(always)]
        fn set(&$self, cx: &mut ::js::context::JSContext, key: Self::Key, value: Self::Value) {
            $self.$internal.set(cx, key, value)
        }

        #[inline(always)]
        fn has(&$self, cx: &mut ::js::context::JSContext, key: Self::Key) -> bool {
            $self.$internal.has(cx, key)
        }

        #[inline(always)]
        fn clear(&$self, cx: &mut ::js::context::JSContext) {
            $self.$internal.clear(cx)
        }

        #[inline(always)]
        fn delete(&$self, cx: &mut ::js::context::JSContext, key: Self::Key) -> bool {
            $self.$internal.delete(cx, key)
        }
    };
}
