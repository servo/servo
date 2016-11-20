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
            element.get_string_attribute(&local_name!($htmlname))
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
            element.has_attribute(&local_name!($htmlname))
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
            element.set_int_attribute(&local_name!($htmlname), value);
            Ok(())
        }
    );
);

#[macro_export]
macro_rules! make_int_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: i32) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;

            let element = self.upcast::<Element>();
            element.set_int_attribute(&local_name!($htmlname), value)
        }
    );
    ($attr:ident, $htmlname:tt) => {
        make_int_setter!($attr, $htmlname, 0);
    };
);

#[macro_export]
macro_rules! make_int_getter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self) -> i32 {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_int_attribute(&local_name!($htmlname), $default)
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
            element.get_uint_attribute(&local_name!($htmlname), $default)
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
            element.get_url_attribute(&local_name!($htmlname))
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
            let url = element.get_url_attribute(&local_name!($htmlname));
            if url.is_empty() {
                let window = window_from_node(self);
                DOMString::from(window.get_url().into_string())
            } else {
                url
            }
        }
    );
);

#[macro_export]
macro_rules! make_string_or_document_url_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            use dom::node::document_from_node;
            let element = self.upcast::<Element>();
            let val = element.get_string_attribute(&local_name!($htmlname));

            if val.is_empty() {
                let doc = document_from_node(self);
                DOMString::from(doc.url().into_string())
            } else {
                val
            }
        }
    );
);

#[macro_export]
macro_rules! make_enumerated_getter(
    ( $attr:ident, $htmlname:tt, $default:expr, $($choices: pat)|+) => (
        fn $attr(&self) -> DOMString {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            use std::ascii::AsciiExt;
            let element = self.upcast::<Element>();
            let mut val = element.get_string_attribute(&local_name!($htmlname));
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
            element.set_string_attribute(&local_name!($htmlname), value)
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
            element.set_bool_attribute(&local_name!($htmlname), value)
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
            let value = AttrValue::from_url(document_from_node(self).url(),
                                            value.into());
            let element = self.upcast::<Element>();
            element.set_attribute(&local_name!($htmlname), value);
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
            element.set_uint_attribute(&local_name!($htmlname), value)
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
            element.set_uint_attribute(&local_name!($htmlname), value);
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
            element.set_atomic_attribute(&local_name!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_legacy_color_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use dom::bindings::inheritance::Castable;
            use dom::element::Element;
            use style::attr::AttrValue;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_legacy_color(value.into());
            element.set_attribute(&local_name!($htmlname), value)
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
            let value = AttrValue::from_dimension(value.into());
            element.set_attribute(&local_name!($htmlname), value)
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
            let value = AttrValue::from_nonzero_dimension(value.into());
            element.set_attribute(&local_name!($htmlname), value)
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

macro_rules! define_window_owned_event_handler(
    ($handler: ident, $event_type: ident, $getter: ident, $setter: ident) => (
        fn $getter(&self) -> Option<::std::rc::Rc<$handler>> {
            window_from_node(self).$getter()
        }

        fn $setter(&self, listener: Option<::std::rc::Rc<$handler>>) {
            window_from_node(self).$setter(listener)
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

macro_rules! beforeunload_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(OnBeforeUnloadEventHandlerNonNull, $event_type,
                              $getter, $setter, set_beforeunload_event_handler);
    )
);

macro_rules! window_owned_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_window_owned_event_handler!(EventHandlerNonNull,
                                           $event_type, $getter, $setter);
    )
);

macro_rules! window_owned_beforeunload_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_window_owned_event_handler!(OnBeforeUnloadEventHandlerNonNull,
                                           $event_type, $getter, $setter);
    )
);

// https://html.spec.whatwg.org/multipage/#globaleventhandlers
// see webidls/EventHandler.webidl
// As more methods get added, just update them here.
macro_rules! global_event_handlers(
    () => (
        event_handler!(blur, GetOnblur, SetOnblur);
        event_handler!(focus, GetOnfocus, SetOnfocus);
        event_handler!(load, GetOnload, SetOnload);
        event_handler!(resize, GetOnresize, SetOnresize);
        event_handler!(scroll, GetOnscroll, SetOnscroll);
        global_event_handlers!(NoOnload);

    );
    (NoOnload) => (
        event_handler!(abort, GetOnabort, SetOnabort);
        event_handler!(cancel, GetOncancel, SetOncancel);
        event_handler!(canplay, GetOncanplay, SetOncanplay);
        event_handler!(canplaythrough, GetOncanplaythrough, SetOncanplaythrough);
        event_handler!(change, GetOnchange, SetOnchange);
        event_handler!(click, GetOnclick, SetOnclick);
        event_handler!(close, GetOnclose, SetOnclose);
        event_handler!(contextmenu, GetOncontextmenu, SetOncontextmenu);
        event_handler!(cuechange, GetOncuechange, SetOncuechange);
        event_handler!(dblclick, GetOndblclick, SetOndblclick);
        event_handler!(drag, GetOndrag, SetOndrag);
        event_handler!(dragend, GetOndragend, SetOndragend);
        event_handler!(dragenter, GetOndragenter, SetOndragenter);
        event_handler!(dragexit, GetOndragexit, SetOndragexit);
        event_handler!(dragleave, GetOndragleave, SetOndragleave);
        event_handler!(dragover, GetOndragover, SetOndragover);
        event_handler!(dragstart, GetOndragstart, SetOndragstart);
        event_handler!(drop, GetOndrop, SetOndrop);
        event_handler!(durationchange, GetOndurationchange, SetOndurationchange);
        event_handler!(emptied, GetOnemptied, SetOnemptied);
        event_handler!(ended, GetOnended, SetOnended);
        error_event_handler!(error, GetOnerror, SetOnerror);
        event_handler!(input, GetOninput, SetOninput);
        event_handler!(invalid, GetOninvalid, SetOninvalid);
        event_handler!(keydown, GetOnkeydown, SetOnkeydown);
        event_handler!(keypress, GetOnkeypress, SetOnkeypress);
        event_handler!(keyup, GetOnkeyup, SetOnkeyup);
        event_handler!(loadeddata, GetOnloadeddata, SetOnloadeddata);
        event_handler!(loadedmetata, GetOnloadedmetadata, SetOnloadedmetadata);
        event_handler!(loadstart, GetOnloadstart, SetOnloadstart);
        event_handler!(mousedown, GetOnmousedown, SetOnmousedown);
        event_handler!(mouseenter, GetOnmouseenter, SetOnmouseenter);
        event_handler!(mouseleave, GetOnmouseleave, SetOnmouseleave);
        event_handler!(mousemove, GetOnmousemove, SetOnmousemove);
        event_handler!(mouseout, GetOnmouseout, SetOnmouseout);
        event_handler!(mouseover, GetOnmouseover, SetOnmouseover);
        event_handler!(mouseup, GetOnmouseup, SetOnmouseup);
        event_handler!(wheel, GetOnwheel, SetOnwheel);
        event_handler!(pause, GetOnpause, SetOnpause);
        event_handler!(play, GetOnplay, SetOnplay);
        event_handler!(playing, GetOnplaying, SetOnplaying);
        event_handler!(progress, GetOnprogress, SetOnprogress);
        event_handler!(ratechange, GetOnratechange, SetOnratechange);
        event_handler!(reset, GetOnreset, SetOnreset);
        event_handler!(seeked, GetOnseeked, SetOnseeked);
        event_handler!(seeking, GetOnseeking, SetOnseeking);
        event_handler!(select, GetOnselect, SetOnselect);
        event_handler!(show, GetOnshow, SetOnshow);
        event_handler!(stalled, GetOnstalled, SetOnstalled);
        event_handler!(submit, GetOnsubmit, SetOnsubmit);
        event_handler!(suspend, GetOnsuspend, SetOnsuspend);
        event_handler!(timeupdate, GetOntimeupdate, SetOntimeupdate);
        event_handler!(toggle, GetOntoggle, SetOntoggle);
        event_handler!(transitionend, GetOntransitionend, SetOntransitionend);
        event_handler!(volumechange, GetOnvolumechange, SetOnvolumechange);
        event_handler!(waiting, GetOnwaiting, SetOnwaiting);
    )
);

// https://html.spec.whatwg.org/multipage/#windoweventhandlers
// see webidls/EventHandler.webidl
// As more methods get added, just update them here.
macro_rules! window_event_handlers(
    () => (
        event_handler!(afterprint, GetOnafterprint, SetOnafterprint);
        event_handler!(beforeprint, GetOnbeforeprint, SetOnbeforeprint);
        beforeunload_event_handler!(beforeunload, GetOnbeforeunload,
                                    SetOnbeforeunload);
        event_handler!(hashchange, GetOnhashchange, SetOnhashchange);
        event_handler!(languagechange, GetOnlanguagechange,
                       SetOnlanguagechange);
        event_handler!(message, GetOnmessage, SetOnmessage);
        event_handler!(offline, GetOnoffline, SetOnoffline);
        event_handler!(online, GetOnonline, SetOnonline);
        event_handler!(pagehide, GetOnpagehide, SetOnpagehide);
        event_handler!(pageshow, GetOnpageshow, SetOnpageshow);
        event_handler!(popstate, GetOnpopstate, SetOnpopstate);
        event_handler!(rejectionhandled, GetOnrejectionhandled,
                       SetOnrejectionhandled);
        event_handler!(storage, GetOnstorage, SetOnstorage);
        event_handler!(unhandledrejection, GetOnunhandledrejection,
                       SetOnunhandledrejection);
        event_handler!(unload, GetOnunload, SetOnunload);
    );
    (ForwardToWindow) => (
        window_owned_event_handler!(afterprint, GetOnafterprint,
                                    SetOnafterprint);
        window_owned_event_handler!(beforeprint, GetOnbeforeprint,
                                    SetOnbeforeprint);
        window_owned_beforeunload_event_handler!(beforeunload,
                                                 GetOnbeforeunload,
                                                 SetOnbeforeunload);
        window_owned_event_handler!(hashchange, GetOnhashchange,
                                    SetOnhashchange);
        window_owned_event_handler!(languagechange, GetOnlanguagechange,
                                    SetOnlanguagechange);
        window_owned_event_handler!(message, GetOnmessage, SetOnmessage);
        window_owned_event_handler!(offline, GetOnoffline, SetOnoffline);
        window_owned_event_handler!(online, GetOnonline, SetOnonline);
        window_owned_event_handler!(pagehide, GetOnpagehide, SetOnpagehide);
        window_owned_event_handler!(pageshow, GetOnpageshow, SetOnpageshow);
        window_owned_event_handler!(popstate, GetOnpopstate, SetOnpopstate);
        window_owned_event_handler!(rejectionhandled, GetOnrejectionhandled,
                                    SetOnrejectionhandled);
        window_owned_event_handler!(storage, GetOnstorage, SetOnstorage);
        window_owned_event_handler!(unhandledrejection, GetOnunhandledrejection,
                                    SetOnunhandledrejection);
        window_owned_event_handler!(unload, GetOnunload, SetOnunload);
    );
);

// https://html.spec.whatwg.org/multipage/#documentandelementeventhandlers
// see webidls/EventHandler.webidl
// As more methods get added, just update them here.
macro_rules! document_and_element_event_handlers(
    () => (
        event_handler!(cut, GetOncut, SetOncut);
        event_handler!(copy, GetOncopy, SetOncopy);
        event_handler!(paste, GetOnpaste, SetOnpaste);
    )
);

#[macro_export]
macro_rules! rooted_vec {
    (let mut $name:ident) => {
        rooted_vec!(let mut $name <- ::std::iter::empty())
    };
    (let $name:ident <- $iter:expr) => {
        let mut __root = $crate::dom::bindings::trace::RootableVec::new_unrooted();
        let $name = $crate::dom::bindings::trace::RootedVec::new(&mut __root, $iter);
    };
    (let mut $name:ident <- $iter:expr) => {
        let mut __root = $crate::dom::bindings::trace::RootableVec::new_unrooted();
        let mut $name = $crate::dom::bindings::trace::RootedVec::new(&mut __root, $iter);
    }
}
