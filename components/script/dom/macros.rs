/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![macro_escape]

#[macro_export]
macro_rules! make_getter(
    ( $attr:ident ) => (
        fn $attr(self) -> DOMString {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use std::ascii::StrAsciiExt;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.get_string_attribute(stringify!($attr).to_ascii_lower().as_slice())
        }
    );
)

#[macro_export]
macro_rules! make_bool_getter(
    ( $attr:ident ) => (
        fn $attr(self) -> bool {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use std::ascii::StrAsciiExt;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.has_attribute(stringify!($attr).to_ascii_lower().as_slice())
        }
    );
)

#[macro_export]
macro_rules! make_uint_getter(
    ( $attr:ident ) => (
        fn $attr(self) -> u32 {
            use dom::element::{Element, AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use std::ascii::StrAsciiExt;
            let element: JSRef<Element> = ElementCast::from_ref(self);
            element.get_uint_attribute(stringify!($attr).to_ascii_lower().as_slice())
        }
    );
)
