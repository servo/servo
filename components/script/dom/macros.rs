/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! make_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            element.get_string_attribute(&Atom::from_slice($htmlname))
        }
    );
    ($attr:ident) => {
        make_getter!($attr, to_lower!(stringify!($attr)));
    }
);

#[macro_export]
macro_rules! make_bool_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> bool {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.has_attribute(&Atom::from_slice($htmlname))
        }
    );
    ($attr:ident) => {
        make_bool_getter!($attr, to_lower!(stringify!($attr)));
    }
);

#[macro_export]
macro_rules! make_uint_getter(
    ($attr:ident, $htmlname:expr, $default:expr) => (
        fn $attr(self) -> u32 {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.get_uint_attribute(&Atom::from_slice($htmlname), $default)
        }
    );
    ($attr:ident, $htmlname:expr) => {
        make_uint_getter!($attr, $htmlname, 0);
    };
    ($attr:ident) => {
        make_uint_getter!($attr, to_lower!(stringify!($attr)));
    }
);

#[macro_export]
macro_rules! make_url_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not runtime.
            element.get_url_attribute(&Atom::from_slice($htmlname))
        }
    );
    ($attr:ident) => {
        // FIXME(pcwalton): Do this at compile time, not runtime.
        make_url_getter!($attr, to_lower!(stringify!($attr)));
    }
);

#[macro_export]
macro_rules! make_url_or_base_getter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            let url = element.get_url_attribute(&Atom::from_slice($htmlname));
            if url.is_empty() {
                let window = window_from_node(self);
                window.r().get_url().serialize()
            } else {
                url
            }
        }
    );
    ($attr:ident) => {
        make_url_or_base_getter!($attr, to_lower!(stringify!($attr)));
    }
);

#[macro_export]
macro_rules! make_enumerated_getter(
    ( $attr:ident, $htmlname:expr, $default:expr, $(($choices: pat))|+) => (
        fn $attr(self) -> DOMString {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use std::ascii::AsciiExt;
            use std::borrow::ToOwned;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            let mut val = element.get_string_attribute(&Atom::from_slice($htmlname));
            val.make_ascii_lowercase();
            // https://html.spec.whatwg.org/multipage/#attr-fs-method
            match &*val {
                $($choices)|+ => val,
                _ => $default.to_owned()
            }
        }
    );
    ($attr:ident, $default:expr, $(($choices: pat))|+) => {
        make_enumerated_getter!($attr, &to_lower!(stringify!($attr)), $default, $(($choices))|+);
    }
);

// concat_idents! doesn't work for function name positions, so
// we have to specify both the content name and the HTML name here
#[macro_export]
macro_rules! make_setter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self, value: DOMString) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not at runtime.
            element.set_string_attribute(&Atom::from_slice($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_bool_setter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self, value: bool) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not at runtime.
            element.set_bool_attribute(&Atom::from_slice($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_uint_setter(
    ($attr:ident, $htmlname:expr, $default:expr) => (
        fn $attr(self, value: u32) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let value = if value > 2147483647 {
                $default
            } else {
                value
            };
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not at runtime.
            element.set_uint_attribute(&Atom::from_slice($htmlname), value)
        }
    );
    ($attr:ident, $htmlname:expr) => {
        make_uint_setter!($attr, $htmlname, 0);
    };
);

#[macro_export]
macro_rules! make_limited_uint_setter(
    ($attr:ident, $htmlname:expr, $default:expr) => (
        fn $attr(self, value: u32) -> $crate::dom::bindings::error::ErrorResult {
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
    ($attr:ident, $htmlname:expr) => {
        make_limited_uint_setter!($attr, $htmlname, 1);
    };
    ($attr:ident) => {
        make_limited_uint_setter!($attr, to_lower!(stringify!($attr)));
    };
);

#[macro_export]
macro_rules! make_atomic_setter(
    ( $attr:ident, $htmlname:expr ) => (
        fn $attr(self, value: DOMString) {
            use dom::bindings::codegen::InheritTypes::ElementCast;
            use string_cache::Atom;
            let element = ElementCast::from_ref(self);
            // FIXME(pcwalton): Do this at compile time, not at runtime.
            element.set_atomic_attribute(&Atom::from_slice($htmlname), value)
        }
    );
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
);

/// These are used to generate a event handler which has no special case.
macro_rules! define_event_handler(
    ($handler: ident, $event_type: ident, $getter: ident, $setter: ident) => (
        fn $getter(self) -> Option<::std::rc::Rc<$handler>> {
            let eventtarget = EventTargetCast::from_ref(self);
            eventtarget.get_event_handler_common(stringify!($event_type))
        }

        fn $setter(self, listener: Option<::std::rc::Rc<$handler>>) {
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
