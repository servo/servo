/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `DOMString` attributes which
/// don't have any special cases.
#[macro_export]
macro_rules! make_getter_setter(
    ($getter:ident, $setter:ident, $htmlname:expr) => (
        fn $getter(&self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.get_string_attribute(&Atom::from_slice($htmlname))
        }
        fn $setter(&self, value: DOMString) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_string_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ($getter:ident, $setter:ident) => {
        make_getter_setter!($getter, $setter, to_lower!(stringify!($getter)));
    }
);

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `boolean` attributes.
#[macro_export]
macro_rules! make_bool_getter_setter(
    ( $getter:ident, $setter:ident, $htmlname:expr ) => (
        fn $getter(&self) -> bool {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.has_attribute(&Atom::from_slice($htmlname))
        }
         fn $setter(&self, value: bool) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_bool_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ( $getter:ident, $setter:ident ) => {
        make_bool_getter_setter!($getter, $setter, to_lower!(stringify!($getter)));
    }
);

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `unsigned long` attributes which
/// don't have any special cases.
///
/// If you use this, you must also add a case to parse_plain_attribute
/// for the attribute in question.
#[macro_export]
macro_rules! make_uint_getter_setter(
    ( $getter:ident, $setter:ident, $htmlname:expr, $default:expr ) => (
        fn $getter(&self) -> u32 {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.get_uint_attribute(&Atom::from_slice($htmlname), $default)
        }
        fn $setter(&self, value: u32) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let value = if value > 2147483647 {
                $default
            } else {
                value
            };
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_uint_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ( $getter:ident, $setter:ident, $htmlname:expr ) => {
        make_uint_getter_setter!($getter, $setter, $htmlname, 0);
    };
    ( $getter:ident, $setter:ident ) => {
        make_uint_getter_setter!($getter, $setter, to_lower!(stringify!($getter)));
    }
);

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `unsigned long` attributes which
/// are limited to only non-negative numbers greater than zero.
///
/// If you use this, you must also add a case to parse_plain_attribute
/// for the attribute in question.
#[macro_export]
macro_rules! make_limited_uint_getter_setter(
    ( $getter:ident, $setter:ident, $htmlname:expr, $default:expr ) => (
        fn $getter(&self) -> u32 {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.get_uint_attribute(&Atom::from_slice($htmlname), $default)
        }
        fn $setter(&self, value: u32) -> $crate::dom::bindings::error::ErrorResult {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let value = if value == 0 {
                return Err($crate::dom::bindings::error::Error::IndexSize);
            } else if value > 2147483647 {
                $default
            } else {
                value
            };
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_uint_attribute(&Atom::from_slice($htmlname), value);
            Ok(())
        }
    );
    ( $getter:ident, $setter:ident, $htmlname:expr ) => {
        make_limited_uint_getter_setter!($getter, $setter, $htmlname, 1);
    };
    ( $getter:ident, $setter:ident ) => {
        make_limited_uint_getter_setter!($getter, $setter, to_lower!(stringify!($getter)));
    }
);

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `DOMString` attributes which
/// don't have any special cases, and are represented as an Atom internally.
///
/// If you use this, you must also add a case to parse_plain_attribute
/// for the attribute in question.
#[macro_export]
macro_rules! make_atomic_getter_setter(
    ( $getter:ident, $setter:ident, $htmlname:expr ) => (
        fn $getter(&self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.get_string_attribute(&Atom::from_slice($htmlname))
        }
        fn $setter(&self, value: DOMString) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_atomic_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ( $getter:ident, $setter:ident ) => {
        make_atomic_getter_setter!($getter, $setter, to_lower!(stringify!($getter)));
    }
);

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `DOMString` attributes whose
/// content attribute is defined to contain a URL.
#[macro_export]
macro_rules! make_url_getter_setter(
    ( $getter:ident, $setter:ident, $htmlname:expr ) => (
        fn $getter(&self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.get_url_attribute(&Atom::from_slice($htmlname))
        }
        fn $setter(&self, value: DOMString) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_string_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ($getter:ident, $setter:ident) => {
        make_url_getter_setter!($getter, $setter, to_lower!(stringify!($getter)));
    }
);

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `DOMString` attributes whose
/// content attribute is defined to contain a URL, except that on getting, if the
/// content attribute is missing or empty, the document's address is returned
/// instead.
#[macro_export]
macro_rules! make_url_or_base_getter_setter(
    ( $getter:ident, $setter:ident, $htmlname:expr ) => (
        fn $getter(&self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            let url = element.get_url_attribute(&Atom::from_slice($htmlname));
            if url.is_empty() {
                let window = window_from_node(self);
                window.r().get_url().serialize()
            } else {
                url
            }
        }
        fn $setter(&self, value: DOMString) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_string_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ($getter:ident, $setter:ident) => {
        make_url_or_base_getter_setter!($getter, $setter, to_lower!(stringify!($getter)));
    }
);

/// Creates a getter and setter for an simple IDL attribute which reflects
/// a content attribute.  This is used for `DOMString` attributes where
/// the content attribute is an enumerated attribute and the IDL attribute
/// is limited to only known values.
#[macro_export]
macro_rules! make_enumerated_getter_setter(
    ( $getter:ident, $setter:ident, $htmlname:expr, $default:expr, $(($choices: pat))|+) => (
        fn $getter(&self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use std::ascii::AsciiExt;
            use std::borrow::ToOwned;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            let mut val = element.get_string_attribute(&Atom::from_slice($htmlname));
            val.make_ascii_lowercase();
            // https://html.spec.whatwg.org/multipage/#attr-fs-method
            match &*val {
                $($choices)|+ => val,
                _ => $default.to_owned()
            }
        }
        fn $setter(&self, value: DOMString) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.set_string_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ($getter:ident, $setter:ident, $default:expr, $(($choices: pat))|+) => {
        make_enumerated_getter_setter!($getter, $setter, &to_lower!(stringify!($getter)), $default, $(($choices))|+);
    }
);

/// For use on non-jsmanaged types
/// Use #[derive(JSTraceable)] on JS managed types
macro_rules! no_jsmanaged_fields(
    ($($ty:ident),+) => (
        $(
            impl $crate::dom::bindings::trace::JSTraceable for $ty {
                #[inline]
                fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                    // Do nothing
                }
            }
        )+
    );
    ($ty:ident<$($gen:ident),+>) => (
        impl<$($gen),+> $crate::dom::bindings::trace::JSTraceable for $ty<$($gen),+> {
            #[inline]
            fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                // Do nothing
            }
        }
    );
    ($ty:ident<$($gen:ident: $bound:ident),+>) => (
        impl<$($gen: $bound),+> $crate::dom::bindings::trace::JSTraceable for $ty<$($gen),+> {
            #[inline]
            fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                // Do nothing
            }
        }
    );
);

/// These are used to generate a event handler which has no special case.
macro_rules! define_event_handler(
    ($handler: ident, $event_type: ident, $getter: ident, $setter: ident) => (
        fn $getter(&self) -> Option<::std::rc::Rc<$handler>> {
            let eventtarget = EventTargetCast::from_ref(self);
            eventtarget.get_event_handler_common(stringify!($event_type))
        }

        fn $setter(&self, listener: Option<::std::rc::Rc<$handler>>) {
            let eventtarget = EventTargetCast::from_ref(self);
            eventtarget.set_event_handler_common(stringify!($event_type), listener)
        }
    )
);

macro_rules! event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(EventHandlerNonNull, $event_type, $getter, $setter);
    )
);

macro_rules! error_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(OnErrorEventHandlerNonNull, $event_type, $getter, $setter);
    )
);

// https://html.spec.whatwg.org/multipage/#globaleventhandlers
// see webidls/EventHandler.webidl
// As more methods get added, just update them here.
macro_rules! global_event_handlers(
    () => (
        event_handler!(load, GetOnload, SetOnload);
        global_event_handlers!(NoOnload);

    );
    (NoOnload) => (
        event_handler!(click, GetOnclick, SetOnclick);
        event_handler!(keydown, GetOnkeydown, SetOnkeydown);
        event_handler!(keypress, GetOnkeypress, SetOnkeypress);
        event_handler!(keyup, GetOnkeyup, SetOnkeyup);
        event_handler!(input, GetOninput, SetOninput);
        event_handler!(change, GetOnchange, SetOnchange);
        event_handler!(submit, GetOnsubmit, SetOnsubmit);
    )
);
