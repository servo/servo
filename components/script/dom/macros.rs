/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! make_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_string_attribute(&local_name!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_bool_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> bool {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.has_attribute(&local_name!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_limited_int_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: i32) -> $crate::dom::bindings::error::ErrorResult {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;

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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;

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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
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
        fn $attr(&self) -> USVString {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_url_attribute(&local_name!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_url_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: USVString) {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.set_url_attribute(&local_name!($htmlname),
                                         value);
        }
    );
);

#[macro_export]
macro_rules! make_form_action_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            let doc = crate::dom::node::document_from_node(self);
            let attr = element.get_attribute(&ns!(), &local_name!($htmlname));
            let value = attr.as_ref().map(|attr| attr.value());
            let value = match value {
                Some(ref value) if !value.is_empty() => &***value,
                _ => return doc.url().into_string().into(),
            };
            match doc.base_url().join(value) {
                Ok(parsed) => parsed.into_string().into(),
                Err(_) => value.to_owned().into(),
            }
        }
    );
);

#[macro_export]
macro_rules! make_enumerated_getter(
    ( $attr:ident, $htmlname:tt, $default:expr, $($choices: pat)|+) => (
        fn $attr(&self) -> DOMString {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.set_string_attribute(&local_name!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_bool_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: bool) {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.set_bool_attribute(&local_name!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_uint_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: u32) {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            use crate::dom::values::UNSIGNED_LONG_MAX;
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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            use crate::dom::values::UNSIGNED_LONG_MAX;
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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.set_atomic_attribute(&local_name!($htmlname), value)
        }
    );
);

#[macro_export]
macro_rules! make_legacy_color_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
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
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::element::Element;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_nonzero_dimension(value.into());
            element.set_attribute(&local_name!($htmlname), value)
        }
    );
);

/// For use on non-jsmanaged types
/// Use #[derive(JSTraceable)] on JS managed types
macro_rules! unsafe_no_jsmanaged_fields(
    ($($ty:ty),+) => (
        $(
            #[allow(unsafe_code)]
            unsafe impl $crate::dom::bindings::trace::JSTraceable for $ty {
                #[inline]
                unsafe fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                    // Do nothing
                }
            }
        )+
    );
);

macro_rules! jsmanaged_array(
    ($count:expr) => (
        #[allow(unsafe_code)]
        unsafe impl<T> $crate::dom::bindings::trace::JSTraceable for [T; $count]
            where T: $crate::dom::bindings::trace::JSTraceable
        {
            #[inline]
            unsafe fn trace(&self, tracer: *mut ::js::jsapi::JSTracer) {
                for v in self.iter() {
                    v.trace(tracer);
                }
            }
        }
    );
);

/// These are used to generate a event handler which has no special case.
macro_rules! define_event_handler(
    ($handler: ty, $event_type: ident, $getter: ident, $setter: ident, $setter_fn: ident) => (
        fn $getter(&self) -> Option<::std::rc::Rc<$handler>> {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::eventtarget::EventTarget;
            let eventtarget = self.upcast::<EventTarget>();
            eventtarget.get_event_handler_common(stringify!($event_type))
        }

        fn $setter(&self, listener: Option<::std::rc::Rc<$handler>>) {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::eventtarget::EventTarget;
            let eventtarget = self.upcast::<EventTarget>();
            eventtarget.$setter_fn(stringify!($event_type), listener)
        }
    )
);

macro_rules! define_window_owned_event_handler(
    ($handler: ty, $event_type: ident, $getter: ident, $setter: ident) => (
        fn $getter(&self) -> Option<::std::rc::Rc<$handler>> {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().$getter()
            } else {
                None
            }
        }

        fn $setter(&self, listener: Option<::std::rc::Rc<$handler>>) {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().$setter(listener)
            }
        }
    )
);

macro_rules! event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(
            crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull,
            $event_type,
            $getter,
            $setter,
            set_event_handler_common
        );
    )
);

macro_rules! error_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(
            crate::dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull,
            $event_type,
            $getter,
            $setter,
            set_error_event_handler
        );
    )
);

macro_rules! beforeunload_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_event_handler!(
            crate::dom::bindings::codegen::Bindings::EventHandlerBinding::OnBeforeUnloadEventHandlerNonNull,
            $event_type,
            $getter,
            $setter,
            set_beforeunload_event_handler
        );
    )
);

macro_rules! window_owned_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_window_owned_event_handler!(
            crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull,
            $event_type,
            $getter,
            $setter
        );
    )
);

macro_rules! window_owned_beforeunload_event_handler(
    ($event_type: ident, $getter: ident, $setter: ident) => (
        define_window_owned_event_handler!(
            crate::dom::bindings::codegen::Bindings::EventHandlerBinding::OnBeforeUnloadEventHandlerNonNull,
            $event_type,
            $getter,
            $setter
        );
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
        event_handler!(loadedmetadata, GetOnloadedmetadata, SetOnloadedmetadata);
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
        event_handler!(messageerror, GetOnmessageerror, SetOnmessageerror);
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
        event_handler!(vrdisplayconnect, GetOnvrdisplayconnect, SetOnvrdisplayconnect);
        event_handler!(vrdisplaydisconnect, GetOnvrdisplaydisconnect, SetOnvrdisplaydisconnect);
        event_handler!(vrdisplayactivate, GetOnvrdisplayactivate, SetOnvrdisplayactivate);
        event_handler!(vrdisplaydeactivate, GetOnvrdisplaydeactivate, SetOnvrdisplaydeactivate);
        event_handler!(vrdisplayblur, GetOnvrdisplayblur, SetOnvrdisplayblur);
        event_handler!(vrdisplayfocus, GetOnvrdisplayfocus, SetOnvrdisplayfocus);
        event_handler!(vrdisplaypresentchange, GetOnvrdisplaypresentchange, SetOnvrdisplaypresentchange);
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
        window_owned_event_handler!(messageerror, GetOnmessageerror, SetOnmessageerror);
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

        window_owned_event_handler!(vrdisplayconnect, GetOnvrdisplayconnect, SetOnvrdisplayconnect);
        window_owned_event_handler!(vrdisplaydisconnect, GetOnvrdisplaydisconnect, SetOnvrdisplaydisconnect);
        window_owned_event_handler!(vrdisplayactivate, GetOnvrdisplayactivate, SetOnvrdisplayactivate);
        window_owned_event_handler!(vrdisplaydeactivate, GetOnvrdisplaydeactivate, SetOnvrdisplaydeactivate);
        window_owned_event_handler!(vrdisplayblur, GetOnvrdisplayblur, SetOnvrdisplayblur);
        window_owned_event_handler!(vrdisplayfocus, GetOnvrdisplayfocus, SetOnvrdisplayfocus);
        window_owned_event_handler!(vrdisplaypresentchange, GetOnvrdisplaypresentchange, SetOnvrdisplaypresentchange);
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
        let mut root = $crate::dom::bindings::trace::RootableVec::new_unrooted();
        let mut $name = $crate::dom::bindings::trace::RootedVec::new(&mut root);
    };
    (let $name:ident <- $iter:expr) => {
        let mut root = $crate::dom::bindings::trace::RootableVec::new_unrooted();
        let $name = $crate::dom::bindings::trace::RootedVec::from_iter(&mut root, $iter);
    };
    (let mut $name:ident <- $iter:expr) => {
        let mut root = $crate::dom::bindings::trace::RootableVec::new_unrooted();
        let mut $name = $crate::dom::bindings::trace::RootedVec::from_iter(&mut root, $iter);
    };
}

/// DOM struct implementation for simple interfaces inheriting from PerformanceEntry.
macro_rules! impl_performance_entry_struct(
    ($binding:ident, $struct:ident, $type:expr) => (
        use crate::dom::bindings::codegen::Bindings::$binding;
        use crate::dom::bindings::reflector::reflect_dom_object;
        use crate::dom::bindings::root::DomRoot;
        use crate::dom::bindings::str::DOMString;
        use crate::dom::globalscope::GlobalScope;
        use crate::dom::performanceentry::PerformanceEntry;
        use dom_struct::dom_struct;

        #[dom_struct]
        pub struct $struct {
            entry: PerformanceEntry,
        }

        impl $struct {
            fn new_inherited(name: DOMString, start_time: f64, duration: f64)
                -> $struct {
                $struct {
                    entry: PerformanceEntry::new_inherited(name,
                                                           DOMString::from($type),
                                                           start_time,
                                                           duration)
                }
            }

            #[allow(unrooted_must_root)]
            pub fn new(global: &GlobalScope,
                       name: DOMString,
                       start_time: f64,
                       duration: f64) -> DomRoot<$struct> {
                let entry = $struct::new_inherited(name, start_time, duration);
                reflect_dom_object(Box::new(entry), global, $binding::Wrap)
            }
        }
    );
);

macro_rules! handle_potential_webgl_error {
    ($context:expr, $call:expr, $return_on_error:expr) => {
        match $call {
            Ok(ret) => ret,
            Err(error) => {
                $context.webgl_error(error);
                $return_on_error
            },
        }
    };
    ($context:expr, $call:expr) => {
        handle_potential_webgl_error!($context, $call, ());
    };
}

macro_rules! impl_document_or_shadow_root_helpers(
    () => (
        /// <https://html.spec.whatwg.org/multipage/#concept-document-bc>
        #[inline]
        pub fn browsing_context(&self) -> Option<DomRoot<WindowProxy>> {
            if self.has_browsing_context {
                self.window.undiscarded_window_proxy()
            } else {
                None
            }
        }

        pub fn nodes_from_point(
            &self,
            client_point: &Point2D<f32>,
            reflow_goal: NodesFromPointQueryType,
        ) -> Vec<UntrustedNodeAddress> {
            if !self
                .window
                .layout_reflow(QueryMsg::NodesFromPointQuery(*client_point, reflow_goal))
            {
                return vec![];
            };

            self.window.layout().nodes_from_point_response()
        }
    );
);

macro_rules! impl_document_or_shadow_root_methods(
    ($struct:ident) => (
        #[allow(unsafe_code)]
        // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
        fn ElementFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Option<DomRoot<Element>> {
            let x = *x as f32;
            let y = *y as f32;
            let point = &Point2D::new(x, y);
            let viewport = self.window.window_size().initial_viewport;

            if self.browsing_context().is_none() {
                return None;
            }

            if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
                return None;
            }

            match self
                .nodes_from_point(point, NodesFromPointQueryType::Topmost)
                .first()
            {
                Some(address) => {
                    let js_runtime = unsafe { JS_GetRuntime(self.window.get_cx()) };
                    let node = unsafe { node::from_untrusted_node_address(js_runtime, *address) };
                    let parent_node = node.GetParentNode().unwrap();
                    let element_ref = node
                        .downcast::<Element>()
                        .unwrap_or_else(|| parent_node.downcast::<Element>().unwrap());

                    Some(DomRoot::from_ref(element_ref))
                },
                None => self.GetDocumentElement(),
            }
        }

        #[allow(unsafe_code)]
        // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
        fn ElementsFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Vec<DomRoot<Element>> {
            let x = *x as f32;
            let y = *y as f32;
            let point = &Point2D::new(x, y);
            let viewport = self.window.window_size().initial_viewport;

            if self.browsing_context().is_none() {
                return vec![];
            }

            // Step 2
            if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
                return vec![];
            }

            let js_runtime = unsafe { JS_GetRuntime(self.window.get_cx()) };

            // Step 1 and Step 3
            let nodes = self.nodes_from_point(point, NodesFromPointQueryType::All);
            let mut elements: Vec<DomRoot<Element>> = nodes
                .iter()
                .flat_map(|&untrusted_node_address| {
                    let node = unsafe {
                        node::from_untrusted_node_address(js_runtime, untrusted_node_address)
                    };
                    DomRoot::downcast::<Element>(node)
                })
                .collect();

            // Step 4
            if let Some(root_element) = self.GetDocumentElement() {
                if elements.last() != Some(&root_element) {
                    elements.push(root_element);
                }
            }

            // Step 5
            elements
        }

        // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
        fn GetActiveElement(&self) -> Option<DomRoot<Element>> {
            // TODO: Step 2.

            match self.get_focused_element() {
                Some(element) => Some(element), // Step 3. and 4.
                None => match self.GetBody() {
                    // Step 5.
                    Some(body) => Some(DomRoot::upcast(body)),
                    None => self.GetDocumentElement(),
                },
            }
        }

        // https://drafts.csswg.org/cssom/#dom-document-stylesheets
        fn StyleSheets(&self) -> DomRoot<StyleSheetList> {
            self.stylesheet_list
                .or_init(|| StyleSheetList::new(&self.window, DocumentOrShadowRoot::$struct(Dom::from_ref(self))))
        }
    );
);
