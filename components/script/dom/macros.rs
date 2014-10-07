/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! make_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> DOMString {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            #[allow(unused_imports)]
            use std::ascii::StrAsciiExt;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.get_string_attribute($htmlname)
        }
    );
    ($attr:ident) => {
        make_getter!($attr, stringify!($attr).to_ascii_lower().as_slice())
    }
)

#[macro_export]
macro_rules! make_bool_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> bool {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            #[allow(unused_imports)]
            use std::ascii::StrAsciiExt;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.has_attribute($htmlname)
        }
    );
    ($attr:ident) => {
        make_bool_getter!($attr, stringify!($attr).to_ascii_lower().as_slice())
    }
)

#[macro_export]
macro_rules! make_uint_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> u32 {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            #[allow(unused_imports)]
            use std::ascii::StrAsciiExt;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.get_uint_attribute($htmlname)
        }
    );
    ($attr:ident) => {
        make_uint_getter!($attr, stringify!($attr).to_ascii_lower().as_slice())
    }
)

#[macro_export]
macro_rules! make_url_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> DOMString {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            #[allow(unused_imports)]
            use std::ascii::StrAsciiExt;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.get_url_attribute($htmlname)
        }
    );
    ($attr:ident) => {
        make_url_getter!($attr, stringify!($attr).to_ascii_lower().as_slice())
    }
)

// concat_idents! doesn't work for function name positions, so
// we have to specify both the content name and the HTML name here
#[macro_export]
macro_rules! make_setter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self, value: DOMString) {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.set_string_attribute($htmlname, value)
        }
    );
)

#[macro_export]
macro_rules! make_bool_setter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self, value: bool) {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.set_bool_attribute($htmlname, value)
        }
    );
)

#[macro_export]
macro_rules! make_uint_setter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self, value: u32) {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.set_uint_attribute($htmlname, value)
        }
    );
)

/// For use on non-jsmanaged types
/// Use #[jstraceable] on JS managed types
macro_rules! untraceable(
    ($($ty:ident),+) => (
        $(
            impl JSTraceable for $ty {
                #[inline]
                fn trace(&self, _: *mut JSTracer) {
                    // Do nothing
                }
            }
        )+
    );
    ($ty:ident<$($gen:ident),+>) => (
        impl<$($gen),+> JSTraceable for $ty<$($gen),+> {
            #[inline]
            fn trace(&self, _: *mut JSTracer) {
                // Do nothing
            }
        }
    );
)

