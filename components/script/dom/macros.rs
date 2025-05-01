/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! make_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_string_attribute(&html5ever::local_name!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_bool_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> bool {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.has_attribute(&html5ever::local_name!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_limited_int_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: i32) -> $crate::dom::bindings::error::ErrorResult {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;

            let value = if value < 0 {
                return Err($crate::dom::bindings::error::Error::IndexSize);
            } else {
                value
            };

            let element = self.upcast::<Element>();
            element.set_int_attribute(&html5ever::local_name!($htmlname), value, CanGc::note());
            Ok(())
        }
    );
);

#[macro_export]
macro_rules! make_int_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: i32) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;

            let element = self.upcast::<Element>();
            element.set_int_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
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
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_int_attribute(&html5ever::local_name!($htmlname), $default)
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
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_uint_attribute(&html5ever::local_name!($htmlname), $default)
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
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_url_attribute(&html5ever::local_name!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_url_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: USVString) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            element.set_url_attribute(&html5ever::local_name!($htmlname),
                                         value, CanGc::note());
        }
    );
);

#[macro_export]
macro_rules! make_trusted_type_url_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> TrustedScriptURLOrUSVString {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            let element = self.upcast::<Element>();
            element.get_trusted_type_url_attribute(&html5ever::local_name!($htmlname))
        }
    );
);

#[macro_export]
macro_rules! make_trusted_type_url_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: TrustedScriptURLOrUSVString, can_gc: CanGc) -> Fallible<()> {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            element.set_trusted_type_url_attribute(&html5ever::local_name!($htmlname),
                                         value, can_gc)
        }
    );
);

#[macro_export]
macro_rules! make_form_action_getter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self) -> DOMString {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            let element = self.upcast::<Element>();
            let doc = $crate::dom::node::NodeTraits::owner_document(self);
            let attr = element.get_attribute(&html5ever::ns!(), &html5ever::local_name!($htmlname));
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
macro_rules! make_labels_getter(
    ( $attr:ident, $memo:ident ) => (
        fn $attr(&self) -> DomRoot<NodeList> {
            use $crate::dom::htmlelement::HTMLElement;
            use $crate::dom::nodelist::NodeList;
            self.$memo.or_init(|| NodeList::new_labels_list(
                self.upcast::<Node>().owner_doc().window(),
                self.upcast::<HTMLElement>(),
                CanGc::note()
                )
            )
        }
    );
);

/// Implements the `To determine the state of an attribute` steps from
/// <https://html.spec.whatwg.org/multipage/#keywords-and-enumerated-attributes>
macro_rules! make_enumerated_getter(
    ($attr:ident,
        $htmlname:tt,
        $($choices:literal)|+,
        missing => $missing:literal,
        invalid => $invalid:literal
    ) => (
        fn $attr(&self) -> DOMString {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::dom::bindings::codegen::Bindings::AttrBinding::Attr_Binding::AttrMethods;

            let attr_or_none = self.upcast::<Element>()
                .get_attribute(&html5ever::ns!(), &html5ever::local_name!($htmlname));
            match attr_or_none  {
                // Step 1. If the attribute is not specified:
                None => {
                    // Step 1.1. If the attribute has a missing value default state defined, then return that
                    // missing value default state.
                    // Step 1.2 Otherwise, return no state.
                    return DOMString::from($missing);
                },
                Some(attr) => {
                    // Step 2. If the attribute's value is an ASCII case-insensitive match for one of the keywords
                    // defined for the attribute, then return the state represented by that keyword.
                    let value: DOMString = attr.Value().to_ascii_lowercase().into();
                    $(
                        if value.str() == $choices {
                            return value;
                        }
                    )+

                    // Step 3. If the attribute has an invalid value default state defined, then return that invalid
                    // value default state.
                    // Step 4. Return no state.
                    return DOMString::from($invalid);
                }
            }
        }
    );
    ($attr:ident,
        $htmlname:tt,
        $($choices:literal)|+,
    ) => (
        make_enumerated_getter!(
            $attr,
            $htmlname,
            $($choices)|+,
            missing => "",
            invalid => ""
        );
    );
    ($attr:ident,
        $htmlname:tt,
        $($choices:literal)|+,
        invalid => $invalid:literal
    ) => (
        make_enumerated_getter!(
            $attr,
            $htmlname,
            $($choices)|+,
            missing => "",
            invalid => $invalid
        );
    );
    ($attr:ident,
        $htmlname:tt,
        $($choices:literal)|+,
        missing => $missing:literal,
    ) => (
        make_enumerated_getter!(
            $attr,
            $htmlname,
            $($choices)|+,
            missing => $missing,
            invalid => ""
        );
    );
);

// concat_idents! doesn't work for function name positions, so
// we have to specify both the content name and the HTML name here
#[macro_export]
macro_rules! make_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            element.set_string_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
        }
    );
);

#[macro_export]
macro_rules! make_bool_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: bool) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            element.set_bool_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
        }
    );
);

#[macro_export]
macro_rules! make_uint_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: u32) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::dom::values::UNSIGNED_LONG_MAX;
            use $crate::script_runtime::CanGc;
            let value = if value > UNSIGNED_LONG_MAX {
                $default
            } else {
                value
            };
            let element = self.upcast::<Element>();
            element.set_uint_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
        }
    );
    ($attr:ident, $htmlname:tt) => {
        make_uint_setter!($attr, $htmlname, 0);
    };
);

#[macro_export]
macro_rules! make_clamped_uint_setter(
    ($attr:ident, $htmlname:tt, $min:expr, $max:expr, $default:expr) => (
        fn $attr(&self, value: u32) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::dom::values::UNSIGNED_LONG_MAX;
            use $crate::script_runtime::CanGc;
            let value = if value > UNSIGNED_LONG_MAX {
                $default
            } else {
                value.clamp($min, $max)
            };

            let element = self.upcast::<Element>();
            element.set_uint_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
        }
    );
);

#[macro_export]
macro_rules! make_limited_uint_setter(
    ($attr:ident, $htmlname:tt, $default:expr) => (
        fn $attr(&self, value: u32) -> $crate::dom::bindings::error::ErrorResult {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::dom::values::UNSIGNED_LONG_MAX;
            use $crate::script_runtime::CanGc;
            let value = if value == 0 {
                return Err($crate::dom::bindings::error::Error::IndexSize);
            } else if value > UNSIGNED_LONG_MAX {
                $default
            } else {
                value
            };
            let element = self.upcast::<Element>();
            element.set_uint_attribute(&html5ever::local_name!($htmlname), value, CanGc::note());
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
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            element.set_atomic_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
        }
    );
);

#[macro_export]
macro_rules! make_legacy_color_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use style::attr::AttrValue;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_legacy_color(value.into());
            element.set_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
        }
    );
);

#[macro_export]
macro_rules! make_dimension_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_dimension(value.into());
            element.set_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
        }
    );
);

#[macro_export]
macro_rules! make_nonzero_dimension_setter(
    ( $attr:ident, $htmlname:tt ) => (
        fn $attr(&self, value: DOMString) {
            use $crate::dom::bindings::inheritance::Castable;
            use $crate::dom::element::Element;
            use $crate::script_runtime::CanGc;
            let element = self.upcast::<Element>();
            let value = AttrValue::from_nonzero_dimension(value.into());
            element.set_attribute(&html5ever::local_name!($htmlname), value, CanGc::note())
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

/// These are used to generate a event handler which has no special case.
macro_rules! define_event_handler(
    ($handler: ty, $event_type: ident, $getter: ident, $setter: ident, $setter_fn: ident) => (
        fn $getter(&self) -> Option<::std::rc::Rc<$handler>> {
            use crate::dom::bindings::inheritance::Castable;
            use crate::dom::eventtarget::EventTarget;
            use crate::script_runtime::CanGc;
            let eventtarget = self.upcast::<EventTarget>();
            eventtarget.get_event_handler_common(stringify!($event_type), CanGc::note())
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
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().$getter()
            } else {
                None
            }
        }

        fn $setter(&self, listener: Option<::std::rc::Rc<$handler>>) {
            let document = self.owner_document();
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
        // These are special when on body/frameset elements
        event_handler!(blur, GetOnblur, SetOnblur);
        error_event_handler!(error, GetOnerror, SetOnerror);
        event_handler!(focus, GetOnfocus, SetOnfocus);
        event_handler!(load, GetOnload, SetOnload);
        event_handler!(resize, GetOnresize, SetOnresize);
        event_handler!(scroll, GetOnscroll, SetOnscroll);
        global_event_handlers!(NoOnload);

    );
    (NoOnload) => (
        event_handler!(abort, GetOnabort, SetOnabort);
        event_handler!(animationend, GetOnanimationend, SetOnanimationend);
        event_handler!(animationiteration, GetOnanimationiteration, SetOnanimationiteration);
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
        event_handler!(formdata, GetOnformdata, SetOnformdata);
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
        event_handler!(securitypolicyviolation, GetOnsecuritypolicyviolation, SetOnsecuritypolicyviolation);
        event_handler!(seeked, GetOnseeked, SetOnseeked);
        event_handler!(seeking, GetOnseeking, SetOnseeking);
        event_handler!(select, GetOnselect, SetOnselect);
        event_handler!(selectionchange, GetOnselectionchange, SetOnselectionchange);
        event_handler!(selectstart, GetOnselectstart, SetOnselectstart);
        event_handler!(show, GetOnshow, SetOnshow);
        event_handler!(stalled, GetOnstalled, SetOnstalled);
        event_handler!(submit, GetOnsubmit, SetOnsubmit);
        event_handler!(suspend, GetOnsuspend, SetOnsuspend);
        event_handler!(timeupdate, GetOntimeupdate, SetOntimeupdate);
        event_handler!(toggle, GetOntoggle, SetOntoggle);
        event_handler!(transitioncancel, GetOntransitioncancel, SetOntransitioncancel);
        event_handler!(transitionend, GetOntransitionend, SetOntransitionend);
        event_handler!(transitionrun, GetOntransitionrun, SetOntransitionrun);
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
        event_handler!(gamepadconnected, GetOngamepadconnected, SetOngamepadconnected);
        event_handler!(gamepaddisconnected, GetOngamepaddisconnected, SetOngamepaddisconnected);
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
        window_owned_event_handler!(gamepadconnected, GetOngamepadconnected, SetOngamepadconnected);
        window_owned_event_handler!(gamepaddisconnected, GetOngamepaddisconnected, SetOngamepaddisconnected);
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

/// DOM struct implementation for simple interfaces inheriting from PerformanceEntry.
macro_rules! impl_performance_entry_struct(
    ($binding:ident, $struct:ident, $type:expr) => (
        use base::cross_process_instant::CrossProcessInstant;
        use time::Duration;

        use crate::dom::bindings::reflector::reflect_dom_object;
        use crate::dom::bindings::root::DomRoot;
        use crate::dom::bindings::str::DOMString;
        use crate::dom::globalscope::GlobalScope;
        use crate::dom::performanceentry::PerformanceEntry;
        use crate::script_runtime::CanGc;
        use dom_struct::dom_struct;

        #[dom_struct]
        pub(crate) struct $struct {
            entry: PerformanceEntry,
        }

        impl $struct {
            fn new_inherited(name: DOMString, start_time: CrossProcessInstant, duration: Duration)
                -> $struct {
                $struct {
                    entry: PerformanceEntry::new_inherited(name,
                                                           DOMString::from($type),
                                                           Some(start_time),
                                                           duration)
                }
            }

            #[cfg_attr(crown, allow(crown::unrooted_must_root))]
            pub(crate) fn new(global: &GlobalScope,
                       name: DOMString,
                       start_time: CrossProcessInstant,
                       duration: Duration) -> DomRoot<$struct> {
                let entry = $struct::new_inherited(name, start_time, duration);
                reflect_dom_object(Box::new(entry), global, CanGc::note())
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
        handle_potential_webgl_error!($context, $call, ())
    };
}
