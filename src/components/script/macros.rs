/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_escape];

macro_rules! make_getters(
    ($cls:ident, [ $($attr:ident),+ ]) => (
        impl $cls {

    $(
        pub fn $attr(&self, abstract_self: &JS<$cls>) -> DOMString {
            use dom::element::{Element,AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::{ElementCast};
            let element: JS<Element> = ElementCast::from(abstract_self);
            element.get_string_attribute(stringify!($attr))
        }
    )+
    }
    );
)

// By default, this will return false for empty attributes
macro_rules! make_bool_getters(
    ($cls:ident, [ $($attr:ident),+ ]) => (
        impl $cls {

    $(
        pub fn $attr(&self, abstract_self: &JS<$cls>) -> bool {
            use dom::element::{Element,AttributeHandlers};
            use dom::bindings::codegen::InheritTypes::{ElementCast};
            let element: JS<Element> = ElementCast::from(abstract_self);
            element.get_bool_attribute(stringify!($attr))
        }
    )+
    }
    );
)
