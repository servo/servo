/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! make_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_string_attribute(&atom!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_bool_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> bool {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.has_attribute(&atom!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_limited_int_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: i32) -> $crate::dom::bindings::error::ErrorResult {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;

            let value = if value < 0 {
                return Err($crate::dom::bindings::error::Error::IndexSize);
            } else {
                value
            };

            let element = self.upcast::<Element>();
            element.set_int_attribute(&atom!($htmlname), value);
            Ok(())
        }
    );
);

#[macro_export]
macro_rules! make_int_getter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self) -> i32 {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_int_attribute(&atom!($htmlname), $default)
        }
    );

    ($attr:ident, $htmlname:tt) => {
        make_int_getter!($attr, $htmlname, 0);
    };
);

#[macro_export]
macro_rules! make_uint_getter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self) -> u32 {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_uint_attribute(&atom!($htmlname), $default)
        }
    );
    ($attr:ident, $htmlname:tt) => {
        make_uint_getter!($attr, $htmlname, 0);
    };
);

#[macro_export]
macro_rules! make_url_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_url_attribute(&atom!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_url_or_base_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            let url = element.get_url_attribute(&atom!($htmlname));
            if url.is_empty() {
                let window = window_from_node(self);
                DOMString::from(window.get_url().serialize())
            } else {
                url
            }
        }
    );
);

#[macro_export]
macro_rules! make_enumerated_getter(
    ( $attr:ident, $htmlname:tt, $default:expr, $(($choices: pat))|+) => (
        fn $attr(&self) -> DOMString {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            use std::ascii::AsciiExt;
            let element = self.upcast::<Element>();
            let mut val = element.get_string_attribute(&atom!($htmlname));
            val.make_ascii_lowercase();
            // https://html.spec.whatwg.org/multipage/#attr-fs-method
            match &*val {
                $($choices)|+ => val,
                _ => DOMString::from($default)
            }
        }
    );
);

// concat_idents! doesn't work for function name positions, so
// we have to specify both the content name and the HTML name here
#[macro_export]
macro_rules! make_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.set_string_attribute(&atom!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_bool_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: bool) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.set_bool_attribute(&atom!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_url_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            use dom::node::document_from_node;
            let value = AttrValue::from_url(document_from_node(self).url(), value);
            let element = self.upcast::<Element>();
            element.set_attribute(&atom!($htmlname), value);
        }
    );
);

#[macro_export]
macro_rules! make_uint_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: u32) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            use dom::values::UNSIGNED_LONG_MAX;
            let value = if value > UNSIGNED_LONG_MAX {
                $default
            } else {
                value
            };
            let element = self.upcast::<Element>();
            element.set_uint_attribute(&atom!($htmlname), value)
        }
    );
    ($attr:ident, $htmlname:tt) => {
        make_uint_setter!($attr, $htmlname, 0);
    };
);

#[macro_export]
macro_rules! make_limited_uint_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: u32) -> $crate::dom::bindings::error::ErrorResult {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            use dom::values::UNSIGNED_LONG_MAX;
            let value = if value == 0 {
                return Err($crate::dom::bindings::error::Error::IndexSize);
            } else if value > UNSIGNED_LONG_MAX {
                $default
            } else {
                value
            };
            let element = self.upcast::<Element>();
            element.set_uint_attribute(&atom!($htmlname), value);
            Ok(())
        }
    );
    ($attr:ident, $htmlname:tt) => {
        make_limited_uint_setter!($attr, $htmlname, 1);
    };
);

#[macro_export]
macro_rules! make_atomic_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.set_atomic_attribute(&atom!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_legacy_color_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use dom::attr::AttrValue;
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_legacy_color(value);
            element.set_attribute(&atom!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_dimension_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_dimension(value);
            element.set_attribute(&atom!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_nonzero_dimension_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_nonzero_dimension(value);
            element.set_attribute(&atom!($htmlname), value)
        }
    );
);

/// For use on non-jsmanaged types
/// Use #[derive(JSTraceable)] on JS managed types
macro_rules! no_jsmanaged_fields(
    ([$ty:ident; $count:expr]) => (
        impl $crate::dom::bindings::trace::JSTraceable for [$ty; $count] {
            #[inline]
            fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                // Do nothing
            }
        }
    );
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
    ($handler: ident, $event_type: ident, $getter: ident, $setter: ident, $setter_fn: ident) => (
        fn $getter(&self) -> Option<::std::rc::Rc<$handler>> {
            use dom::bindings::inheritance::Castable;
            use dom::eventtarget::EventTarget;
            let eventtarget = self.upcast::<EventTarget>();
            eventtarget.get_event_handler_common(stringify!($event_type))
        }

        fn $setter(&self, listener: Option<::std::rc::Rc<$handler>>) {
            use dom::bindings::inheritance::Castable;
            use dom::eventtarget::EventTarget;
            let eventtarget = self.upcast::<EventTarget>();
            eventtarget.$setter_fn(stringify!($event_type), listener)
        }
    )
);

macro_rules! event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(EventHandlerNonNull, $event_type, $getter, $setter,
                              set_event_handler_common);
    )
);

macro_rules! error_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(OnErrorEventHandlerNonNull, $event_type, $getter, $setter,
                              set_error_event_handler);
    )
);

// https://html.spec.whatwg.org/multipage/#globaleventhandlers
// see webidls/EventHandler.webidl
// As more methods get added, just update them here.
macro_rules! global_event_handlers(
    () => (
        event_handler!(blur, GetOnblur, SetOnblur);
        event_handler!(load, GetOnload, SetOnload);
        event_handler!(resize, GetOnresize, SetOnresize);
        global_event_handlers!(NoOnload);

    );
    (NoOnload) => (
        event_handler!(change, GetOnchange, SetOnchange);
        event_handler!(click, GetOnclick, SetOnclick);
        event_handler!(dblclick, GetOndblclick, SetOndblclick);
        error_event_handler!(error, GetOnerror, SetOnerror);
        event_handler!(input, GetOninput, SetOninput);
        event_handler!(keydown, GetOnkeydown, SetOnkeydown);
        event_handler!(keypress, GetOnkeypress, SetOnkeypress);
        event_handler!(keyup, GetOnkeyup, SetOnkeyup);
        event_handler!(mouseover, GetOnmouseover, SetOnmouseover);
        event_handler!(reset, GetOnreset, SetOnreset);
        event_handler!(submit, GetOnsubmit, SetOnsubmit);
        event_handler!(toggle, GetOntoggle, SetOntoggle);
    )
);
