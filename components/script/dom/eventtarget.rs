/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::default::Default;
use std::ffi::CString;
use std::hash::BuildHasherDefault;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use deny_public_fields::DenyPublicFields;
use dom_struct::dom_struct;
use fnv::FnvHasher;
use js::jsapi::JS_GetFunctionObject;
use js::jsval::JSVal;
use js::rust::wrappers::CompileFunction;
use js::rust::{
    transform_u16_to_source_text, CompileOptionsWrapper, HandleObject, RootedObjectVectorWrapper,
};
use libc::c_char;
use servo_atoms::Atom;
use servo_url::ServoUrl;

use crate::dom::beforeunloadevent::BeforeUnloadEvent;
use crate::dom::bindings::callback::{CallbackContainer, CallbackFunction, ExceptionHandling};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEventMethods;
use crate::dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::{
    EventHandlerNonNull, OnBeforeUnloadEventHandlerNonNull, OnErrorEventHandlerNonNull,
};
use crate::dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use crate::dom::bindings::codegen::Bindings::EventTargetBinding::{
    AddEventListenerOptions, EventListenerOptions, EventTargetMethods,
};
use crate::dom::bindings::codegen::Bindings::NodeBinding::GetRootNodeOptions;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    AddEventListenerOptionsOrBoolean, EventListenerOptionsOrBoolean, EventOrString,
};
use crate::dom::bindings::error::{report_pending_exception, Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{
    reflect_dom_object_with_proto, DomGlobal, DomObject, Reflector,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlformelement::FormControlElementHelpers;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::CanGc;

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum CommonEventHandler {
    EventHandler(#[ignore_malloc_size_of = "Rc"] Rc<EventHandlerNonNull>),

    ErrorEventHandler(#[ignore_malloc_size_of = "Rc"] Rc<OnErrorEventHandlerNonNull>),

    BeforeUnloadEventHandler(#[ignore_malloc_size_of = "Rc"] Rc<OnBeforeUnloadEventHandlerNonNull>),
}

impl CommonEventHandler {
    fn parent(&self) -> &CallbackFunction<crate::DomTypeHolder> {
        match *self {
            CommonEventHandler::EventHandler(ref handler) => &handler.parent,
            CommonEventHandler::ErrorEventHandler(ref handler) => &handler.parent,
            CommonEventHandler::BeforeUnloadEventHandler(ref handler) => &handler.parent,
        }
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ListenerPhase {
    Capturing,
    Bubbling,
}

/// <https://html.spec.whatwg.org/multipage/#internal-raw-uncompiled-handler>
#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
struct InternalRawUncompiledHandler {
    source: DOMString,
    #[no_trace]
    url: ServoUrl,
    line: usize,
}

/// A representation of an event handler, either compiled or uncompiled raw source, or null.
#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
enum InlineEventListener {
    Uncompiled(InternalRawUncompiledHandler),
    Compiled(CommonEventHandler),
    Null,
}

/// Get a compiled representation of this event handler, compiling it from its
/// raw source if necessary.
/// <https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler>
fn get_compiled_handler(
    inline_listener: &RefCell<InlineEventListener>,
    owner: &EventTarget,
    ty: &Atom,
    can_gc: CanGc,
) -> Option<CommonEventHandler> {
    let listener = mem::replace(
        &mut *inline_listener.borrow_mut(),
        InlineEventListener::Null,
    );
    let compiled = match listener {
        InlineEventListener::Null => None,
        InlineEventListener::Uncompiled(handler) => {
            owner.get_compiled_event_handler(handler, ty, can_gc)
        },
        InlineEventListener::Compiled(handler) => Some(handler),
    };
    if let Some(ref compiled) = compiled {
        *inline_listener.borrow_mut() = InlineEventListener::Compiled(compiled.clone());
    }
    compiled
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
enum EventListenerType {
    Additive(#[ignore_malloc_size_of = "Rc"] Rc<EventListener>),
    Inline(RefCell<InlineEventListener>),
}

impl EventListenerType {
    fn get_compiled_listener(
        &self,
        owner: &EventTarget,
        ty: &Atom,
        can_gc: CanGc,
    ) -> Option<CompiledEventListener> {
        match *self {
            EventListenerType::Inline(ref inline) => {
                get_compiled_handler(inline, owner, ty, can_gc).map(CompiledEventListener::Handler)
            },
            EventListenerType::Additive(ref listener) => {
                Some(CompiledEventListener::Listener(listener.clone()))
            },
        }
    }
}

/// A representation of an EventListener/EventHandler object that has previously
/// been compiled successfully, if applicable.
pub(crate) enum CompiledEventListener {
    Listener(Rc<EventListener>),
    Handler(CommonEventHandler),
}

impl CompiledEventListener {
    #[allow(unsafe_code)]
    pub(crate) fn associated_global(&self) -> DomRoot<GlobalScope> {
        let obj = match self {
            CompiledEventListener::Listener(listener) => listener.callback(),
            CompiledEventListener::Handler(CommonEventHandler::EventHandler(handler)) => {
                handler.callback()
            },
            CompiledEventListener::Handler(CommonEventHandler::ErrorEventHandler(handler)) => {
                handler.callback()
            },
            CompiledEventListener::Handler(CommonEventHandler::BeforeUnloadEventHandler(
                handler,
            )) => handler.callback(),
        };
        unsafe { GlobalScope::from_object(obj) }
    }

    // https://html.spec.whatwg.org/multipage/#the-event-handler-processing-algorithm
    pub(crate) fn call_or_handle_event(
        &self,
        object: &EventTarget,
        event: &Event,
        exception_handle: ExceptionHandling,
    ) {
        // Step 3
        match *self {
            CompiledEventListener::Listener(ref listener) => {
                let _ = listener.HandleEvent_(object, event, exception_handle);
            },
            CompiledEventListener::Handler(ref handler) => {
                match *handler {
                    CommonEventHandler::ErrorEventHandler(ref handler) => {
                        if let Some(event) = event.downcast::<ErrorEvent>() {
                            if object.is::<Window>() || object.is::<WorkerGlobalScope>() {
                                let cx = GlobalScope::get_cx();
                                rooted!(in(*cx) let mut error: JSVal);
                                event.Error(cx, error.handle_mut());
                                rooted!(in(*cx) let mut rooted_return_value: JSVal);
                                let return_value = handler.Call_(
                                    object,
                                    EventOrString::String(event.Message()),
                                    Some(event.Filename()),
                                    Some(event.Lineno()),
                                    Some(event.Colno()),
                                    Some(error.handle()),
                                    rooted_return_value.handle_mut(),
                                    exception_handle,
                                );
                                // Step 4
                                if let Ok(()) = return_value {
                                    if rooted_return_value.handle().is_boolean() &&
                                        rooted_return_value.handle().to_boolean()
                                    {
                                        event.upcast::<Event>().PreventDefault();
                                    }
                                }
                                return;
                            }
                        }

                        rooted!(in(*GlobalScope::get_cx()) let mut rooted_return_value: JSVal);
                        let _ = handler.Call_(
                            object,
                            EventOrString::Event(DomRoot::from_ref(event)),
                            None,
                            None,
                            None,
                            None,
                            rooted_return_value.handle_mut(),
                            exception_handle,
                        );
                    },

                    CommonEventHandler::BeforeUnloadEventHandler(ref handler) => {
                        if let Some(event) = event.downcast::<BeforeUnloadEvent>() {
                            // Step 5
                            if let Ok(value) =
                                handler.Call_(object, event.upcast::<Event>(), exception_handle)
                            {
                                let rv = event.ReturnValue();
                                if let Some(v) = value {
                                    if rv.is_empty() {
                                        event.SetReturnValue(v);
                                    }
                                    event.upcast::<Event>().PreventDefault();
                                }
                            }
                        } else {
                            // Step 5, "Otherwise" clause
                            let _ =
                                handler.Call_(object, event.upcast::<Event>(), exception_handle);
                        }
                    },

                    CommonEventHandler::EventHandler(ref handler) => {
                        let cx = GlobalScope::get_cx();
                        rooted!(in(*cx) let mut rooted_return_value: JSVal);
                        if let Ok(()) = handler.Call_(
                            object,
                            event,
                            rooted_return_value.handle_mut(),
                            exception_handle,
                        ) {
                            let value = rooted_return_value.handle();

                            //Step 5
                            let should_cancel = value.is_boolean() && !value.to_boolean();

                            if should_cancel {
                                // FIXME: spec says to set the cancelled flag directly
                                // here, not just to prevent default;
                                // can that ever make a difference?
                                event.PreventDefault();
                            }
                        }
                    },
                }
            },
        }
    }
}

// https://dom.spec.whatwg.org/#concept-event-listener
// (as distinct from https://dom.spec.whatwg.org/#callbackdef-eventlistener)
#[derive(Clone, DenyPublicFields, JSTraceable, MallocSizeOf)]
/// A listener in a collection of event listeners.
struct EventListenerEntry {
    phase: ListenerPhase,
    listener: EventListenerType,
    once: bool,
}

impl std::cmp::PartialEq for EventListenerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase && self.listener == other.listener
    }
}

#[derive(JSTraceable, MallocSizeOf)]
/// A mix of potentially uncompiled and compiled event listeners of the same type.
struct EventListeners(Vec<EventListenerEntry>);

impl Deref for EventListeners {
    type Target = Vec<EventListenerEntry>;
    fn deref(&self) -> &Vec<EventListenerEntry> {
        &self.0
    }
}

impl DerefMut for EventListeners {
    fn deref_mut(&mut self) -> &mut Vec<EventListenerEntry> {
        &mut self.0
    }
}

impl EventListeners {
    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    fn get_inline_listener(
        &self,
        owner: &EventTarget,
        ty: &Atom,
        can_gc: CanGc,
    ) -> Option<CommonEventHandler> {
        for entry in &self.0 {
            if let EventListenerType::Inline(ref inline) = entry.listener {
                // Step 1.1-1.8 and Step 2
                return get_compiled_handler(inline, owner, ty, can_gc);
            }
        }

        // Step 2
        None
    }

    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    fn get_listeners(
        &self,
        phase: Option<ListenerPhase>,
        owner: &EventTarget,
        ty: &Atom,
        can_gc: CanGc,
    ) -> Vec<CompiledEventListener> {
        self.0
            .iter()
            .filter_map(|entry| {
                if phase.is_none() || Some(entry.phase) == phase {
                    // Step 1.1-1.8, 2
                    entry.listener.get_compiled_listener(owner, ty, can_gc)
                } else {
                    None
                }
            })
            .collect()
    }

    fn has_listeners(&self) -> bool {
        // TODO: add, and take into account, a 'removed' field?
        // https://dom.spec.whatwg.org/#event-listener-removed
        !self.0.is_empty()
    }
}

#[dom_struct]
pub(crate) struct EventTarget {
    reflector_: Reflector,
    handlers: DomRefCell<HashMapTracedValues<Atom, EventListeners, BuildHasherDefault<FnvHasher>>>,
}

impl EventTarget {
    pub(crate) fn new_inherited() -> EventTarget {
        EventTarget {
            reflector_: Reflector::new(),
            handlers: DomRefCell::new(Default::default()),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<EventTarget> {
        reflect_dom_object_with_proto(
            Box::new(EventTarget::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    /// Determine if there are any listeners for a given event type.
    /// See <https://github.com/whatwg/dom/issues/453>.
    pub(crate) fn has_listeners_for(&self, type_: &Atom) -> bool {
        match self.handlers.borrow().get(type_) {
            Some(listeners) => listeners.has_listeners(),
            None => false,
        }
    }

    pub(crate) fn get_listeners_for(
        &self,
        type_: &Atom,
        specific_phase: Option<ListenerPhase>,
        can_gc: CanGc,
    ) -> Vec<CompiledEventListener> {
        self.handlers
            .borrow()
            .get(type_)
            .map_or(vec![], |listeners| {
                listeners.get_listeners(specific_phase, self, type_, can_gc)
            })
    }

    pub(crate) fn dispatch_event(&self, event: &Event, can_gc: CanGc) -> EventStatus {
        event.dispatch(self, false, can_gc)
    }

    pub(crate) fn remove_all_listeners(&self) {
        *self.handlers.borrow_mut() = Default::default();
    }

    /// <https://html.spec.whatwg.org/multipage/#event-handler-attributes:event-handlers-11>
    fn set_inline_event_listener(&self, ty: Atom, listener: Option<InlineEventListener>) {
        let mut handlers = self.handlers.borrow_mut();
        let entries = match handlers.entry(ty) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(EventListeners(vec![])),
        };

        let idx = entries
            .iter()
            .position(|entry| matches!(entry.listener, EventListenerType::Inline(_)));

        match idx {
            Some(idx) => match listener {
                // Replace if there's something to replace with,
                // but remove entirely if there isn't.
                Some(listener) => {
                    entries[idx].listener = EventListenerType::Inline(listener.into());
                },
                None => {
                    entries.remove(idx);
                },
            },
            None => {
                if let Some(listener) = listener {
                    entries.push(EventListenerEntry {
                        phase: ListenerPhase::Bubbling,
                        listener: EventListenerType::Inline(listener.into()),
                        once: false,
                    });
                }
            },
        }
    }

    pub(crate) fn remove_listener_if_once(&self, ty: &Atom, listener: &Rc<EventListener>) {
        let mut handlers = self.handlers.borrow_mut();

        let listener = EventListenerType::Additive(listener.clone());
        if let Some(entries) = handlers.get_mut(ty) {
            entries.retain(|e| e.listener != listener || !e.once)
        }
    }

    fn get_inline_event_listener(&self, ty: &Atom, can_gc: CanGc) -> Option<CommonEventHandler> {
        let handlers = self.handlers.borrow();
        handlers
            .get(ty)
            .and_then(|entry| entry.get_inline_listener(self, ty, can_gc))
    }

    /// Store the raw uncompiled event handler for on-demand compilation later.
    /// <https://html.spec.whatwg.org/multipage/#event-handler-attributes:event-handler-content-attributes-3>
    pub(crate) fn set_event_handler_uncompiled(
        &self,
        url: ServoUrl,
        line: usize,
        ty: &str,
        source: DOMString,
    ) {
        let handler = InternalRawUncompiledHandler { source, line, url };
        self.set_inline_event_listener(
            Atom::from(ty),
            Some(InlineEventListener::Uncompiled(handler)),
        );
    }

    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    // step 3
    // While the CanGc argument appears unused, it reflects the fact that the CompileFunction
    // API call can trigger a GC operation.
    #[allow(unsafe_code)]
    fn get_compiled_event_handler(
        &self,
        handler: InternalRawUncompiledHandler,
        ty: &Atom,
        can_gc: CanGc,
    ) -> Option<CommonEventHandler> {
        // Step 3.1
        let element = self.downcast::<Element>();
        let document = match element {
            Some(element) => element.owner_document(),
            None => self.downcast::<Window>().unwrap().Document(),
        };

        // Step 3.2
        if !document.is_scripting_enabled() {
            return None;
        }

        // Step 3.3
        let body: Vec<u16> = handler.source.encode_utf16().collect();

        // Step 3.4 is handler.line

        // Step 3.5
        let form_owner = element
            .and_then(|e| e.as_maybe_form_control())
            .and_then(|f| f.form_owner());

        // Step 3.6 TODO: settings objects not implemented

        // Step 3.7 is written as though we call the parser separately
        // from the compiler; since we just call CompileFunction with
        // source text, we handle parse errors later

        // Step 3.8 TODO: settings objects not implemented
        let window = document.window();
        let _ac = enter_realm(window);

        // Step 3.9

        let name = CString::new(format!("on{}", &**ty)).unwrap();

        // Step 3.9, subsection ParameterList
        const ARG_NAMES: &[*const c_char] = &[c"event".as_ptr()];
        const ERROR_ARG_NAMES: &[*const c_char] = &[
            c"event".as_ptr(),
            c"source".as_ptr(),
            c"lineno".as_ptr(),
            c"colno".as_ptr(),
            c"error".as_ptr(),
        ];
        let is_error = ty == &atom!("error") && self.is::<Window>();
        let args = if is_error { ERROR_ARG_NAMES } else { ARG_NAMES };

        let cx = GlobalScope::get_cx();
        let options = unsafe {
            CompileOptionsWrapper::new(*cx, &handler.url.to_string(), handler.line as u32)
        };

        // Step 3.9, subsection Scope steps 1-6
        let scopechain = RootedObjectVectorWrapper::new(*cx);

        if let Some(element) = element {
            scopechain.append(document.reflector().get_jsobject().get());
            if let Some(form_owner) = form_owner {
                scopechain.append(form_owner.reflector().get_jsobject().get());
            }
            scopechain.append(element.reflector().get_jsobject().get());
        }

        rooted!(in(*cx) let mut handler = unsafe {
            CompileFunction(
                *cx,
                scopechain.handle(),
                options.ptr,
                name.as_ptr(),
                args.len() as u32,
                args.as_ptr(),
                &mut transform_u16_to_source_text(&body),
            )
        });
        if handler.get().is_null() {
            // Step 3.7
            let ar = enter_realm(self);
            // FIXME(#13152): dispatch error event.
            report_pending_exception(cx, false, InRealm::Entered(&ar), can_gc);
            return None;
        }

        // Step 3.10 happens when we drop _ac

        // TODO Step 3.11

        // Step 3.12
        let funobj = unsafe { JS_GetFunctionObject(handler.get()) };
        assert!(!funobj.is_null());
        // Step 1.14
        if is_error {
            Some(CommonEventHandler::ErrorEventHandler(unsafe {
                OnErrorEventHandlerNonNull::new(cx, funobj)
            }))
        } else if ty == &atom!("beforeunload") {
            Some(CommonEventHandler::BeforeUnloadEventHandler(unsafe {
                OnBeforeUnloadEventHandlerNonNull::new(cx, funobj)
            }))
        } else {
            Some(CommonEventHandler::EventHandler(unsafe {
                EventHandlerNonNull::new(cx, funobj)
            }))
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn set_event_handler_common<T: CallbackContainer<crate::DomTypeHolder>>(
        &self,
        ty: &str,
        listener: Option<Rc<T>>,
    ) {
        let cx = GlobalScope::get_cx();

        let event_listener = listener.map(|listener| {
            InlineEventListener::Compiled(CommonEventHandler::EventHandler(unsafe {
                EventHandlerNonNull::new(cx, listener.callback())
            }))
        });
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    #[allow(unsafe_code)]
    pub(crate) fn set_error_event_handler<T: CallbackContainer<crate::DomTypeHolder>>(
        &self,
        ty: &str,
        listener: Option<Rc<T>>,
    ) {
        let cx = GlobalScope::get_cx();

        let event_listener = listener.map(|listener| {
            InlineEventListener::Compiled(CommonEventHandler::ErrorEventHandler(unsafe {
                OnErrorEventHandlerNonNull::new(cx, listener.callback())
            }))
        });
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    #[allow(unsafe_code)]
    pub(crate) fn set_beforeunload_event_handler<T: CallbackContainer<crate::DomTypeHolder>>(
        &self,
        ty: &str,
        listener: Option<Rc<T>>,
    ) {
        let cx = GlobalScope::get_cx();

        let event_listener = listener.map(|listener| {
            InlineEventListener::Compiled(CommonEventHandler::BeforeUnloadEventHandler(unsafe {
                OnBeforeUnloadEventHandlerNonNull::new(cx, listener.callback())
            }))
        });
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    #[allow(unsafe_code)]
    pub(crate) fn get_event_handler_common<T: CallbackContainer<crate::DomTypeHolder>>(
        &self,
        ty: &str,
        can_gc: CanGc,
    ) -> Option<Rc<T>> {
        let cx = GlobalScope::get_cx();
        let listener = self.get_inline_event_listener(&Atom::from(ty), can_gc);
        unsafe {
            listener.map(|listener| {
                CallbackContainer::new(cx, listener.parent().callback_holder().get())
            })
        }
    }

    pub(crate) fn has_handlers(&self) -> bool {
        !self.handlers.borrow().is_empty()
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub(crate) fn fire_event(&self, name: Atom, can_gc: CanGc) -> DomRoot<Event> {
        self.fire_event_with_params(
            name,
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        )
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub(crate) fn fire_bubbling_event(&self, name: Atom, can_gc: CanGc) -> DomRoot<Event> {
        self.fire_event_with_params(
            name,
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            can_gc,
        )
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub(crate) fn fire_cancelable_event(&self, name: Atom, can_gc: CanGc) -> DomRoot<Event> {
        self.fire_event_with_params(
            name,
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            can_gc,
        )
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub(crate) fn fire_bubbling_cancelable_event(
        &self,
        name: Atom,
        can_gc: CanGc,
    ) -> DomRoot<Event> {
        self.fire_event_with_params(
            name,
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            can_gc,
        )
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub(crate) fn fire_event_with_params(
        &self,
        name: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        can_gc: CanGc,
    ) -> DomRoot<Event> {
        let event = Event::new(&self.global(), name, bubbles, cancelable, can_gc);
        event.fire(self, can_gc);
        event
    }
    // https://dom.spec.whatwg.org/#dom-eventtarget-addeventlistener
    pub(crate) fn add_event_listener(
        &self,
        ty: DOMString,
        listener: Option<Rc<EventListener>>,
        options: AddEventListenerOptions,
    ) {
        let listener = match listener {
            Some(l) => l,
            None => return,
        };
        let mut handlers = self.handlers.borrow_mut();
        let entry = match handlers.entry(Atom::from(ty)) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(EventListeners(vec![])),
        };

        let phase = if options.parent.capture {
            ListenerPhase::Capturing
        } else {
            ListenerPhase::Bubbling
        };
        let new_entry = EventListenerEntry {
            phase,
            listener: EventListenerType::Additive(listener),
            once: options.once,
        };
        if !entry.contains(&new_entry) {
            entry.push(new_entry);
        }
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-removeeventlistener
    pub(crate) fn remove_event_listener(
        &self,
        ty: DOMString,
        listener: Option<Rc<EventListener>>,
        options: EventListenerOptions,
    ) {
        let Some(ref listener) = listener else {
            return;
        };
        let mut handlers = self.handlers.borrow_mut();
        let entry = handlers.get_mut(&Atom::from(ty));
        if let Some(entry) = entry {
            let phase = if options.capture {
                ListenerPhase::Capturing
            } else {
                ListenerPhase::Bubbling
            };
            let old_entry = EventListenerEntry {
                phase,
                listener: EventListenerType::Additive(listener.clone()),
                once: false,
            };
            if let Some(position) = entry.iter().position(|e| *e == old_entry) {
                entry.remove(position);
            }
        }
    }

    /// <https://dom.spec.whatwg.org/#get-the-parent>
    pub(crate) fn get_the_parent(&self, event: &Event) -> Option<DomRoot<EventTarget>> {
        if let Some(document) = self.downcast::<Document>() {
            if event.type_() == atom!("load") || !document.has_browsing_context() {
                return None;
            } else {
                return Some(DomRoot::from_ref(document.window().upcast::<EventTarget>()));
            }
        }

        if let Some(shadow_root) = self.downcast::<ShadowRoot>() {
            if event.should_pass_shadow_boundary(shadow_root) {
                let host = shadow_root.Host();
                return Some(DomRoot::from_ref(host.upcast::<EventTarget>()));
            } else {
                return None;
            }
        }

        if let Some(node) = self.downcast::<Node>() {
            // > A node’s get the parent algorithm, given an event, returns the node’s assigned slot,
            // > if node is assigned; otherwise node’s parent.
            return node.assigned_slot().map(DomRoot::upcast).or_else(|| {
                node.GetParentNode()
                    .map(|parent| DomRoot::from_ref(parent.upcast::<EventTarget>()))
            });
        }

        None
    }

    // FIXME: This algorithm operates on "objects", which may not be event targets.
    // All our current use-cases only work on event targets, but this might change in the future
    /// <https://dom.spec.whatwg.org/#retarget>
    pub(crate) fn retarget(&self, b: &Self) -> DomRoot<EventTarget> {
        // To retarget an object A against an object B, repeat these steps until they return an object:
        let mut a = DomRoot::from_ref(self);
        loop {
            // Step 1. If one of the following is true
            // * A is not a node
            // * A’s root is not a shadow root
            // * B is a node and A’s root is a shadow-including inclusive ancestor of B
            let Some(a_node) = a.downcast::<Node>() else {
                return a;
            };
            let a_root = a_node.GetRootNode(&GetRootNodeOptions::empty());
            if !a_root.is::<ShadowRoot>() {
                return a;
            }
            if let Some(b_node) = b.downcast::<Node>() {
                if a_root.is_shadow_including_inclusive_ancestor_of(b_node) {
                    return a;
                }
            }

            // Step 2. Set A to A’s root’s host.
            a = DomRoot::from_ref(
                a_root
                    .downcast::<ShadowRoot>()
                    .unwrap()
                    .Host()
                    .upcast::<EventTarget>(),
            );
        }
    }
}

impl EventTargetMethods<crate::DomTypeHolder> for EventTarget {
    // https://dom.spec.whatwg.org/#dom-eventtarget-eventtarget
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<EventTarget>> {
        Ok(EventTarget::new(global, proto, can_gc))
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-addeventlistener
    fn AddEventListener(
        &self,
        ty: DOMString,
        listener: Option<Rc<EventListener>>,
        options: AddEventListenerOptionsOrBoolean,
    ) {
        self.add_event_listener(ty, listener, options.into())
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-removeeventlistener
    fn RemoveEventListener(
        &self,
        ty: DOMString,
        listener: Option<Rc<EventListener>>,
        options: EventListenerOptionsOrBoolean,
    ) {
        self.remove_event_listener(ty, listener, options.into())
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-dispatchevent
    fn DispatchEvent(&self, event: &Event, can_gc: CanGc) -> Fallible<bool> {
        if event.dispatching() || !event.initialized() {
            return Err(Error::InvalidState);
        }
        event.set_trusted(false);
        Ok(match self.dispatch_event(event, can_gc) {
            EventStatus::Canceled => false,
            EventStatus::NotCanceled => true,
        })
    }
}

impl VirtualMethods for EventTarget {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        None
    }
}

impl From<AddEventListenerOptionsOrBoolean> for AddEventListenerOptions {
    fn from(options: AddEventListenerOptionsOrBoolean) -> Self {
        match options {
            AddEventListenerOptionsOrBoolean::AddEventListenerOptions(options) => options,
            AddEventListenerOptionsOrBoolean::Boolean(capture) => Self {
                parent: EventListenerOptions { capture },
                once: false,
            },
        }
    }
}

impl From<EventListenerOptionsOrBoolean> for EventListenerOptions {
    fn from(options: EventListenerOptionsOrBoolean) -> Self {
        match options {
            EventListenerOptionsOrBoolean::EventListenerOptions(options) => options,
            EventListenerOptionsOrBoolean::Boolean(capture) => Self { capture },
        }
    }
}
