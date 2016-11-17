/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::beforeunloadevent::BeforeUnloadEvent;
use dom::bindings::callback::{CallbackContainer, ExceptionHandling, CallbackFunction};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEventMethods;
use dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnBeforeUnloadEventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::UnionTypes::EventOrString;
use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::bindings::str::DOMString;
use dom::element::Element;
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventdispatcher::{EventStatus, dispatch_event};
use dom::node::document_from_node;
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use fnv::FnvHasher;
use heapsize::HeapSizeOf;
use js::jsapi::{CompileFunction, JS_GetFunctionObject, JSAutoCompartment};
use js::rust::{AutoObjectVectorWrapper, CompileOptionsWrapper};
use libc::{c_char, size_t};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::default::Default;
use std::ffi::CString;
use std::hash::BuildHasherDefault;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::rc::Rc;

#[derive(PartialEq, Clone, JSTraceable)]
pub enum CommonEventHandler {
    EventHandler(Rc<EventHandlerNonNull>),
    ErrorEventHandler(Rc<OnErrorEventHandlerNonNull>),
    BeforeUnloadEventHandler(Rc<OnBeforeUnloadEventHandlerNonNull>),
}

impl CommonEventHandler {
    fn parent(&self) -> &CallbackFunction {
        match *self {
            CommonEventHandler::EventHandler(ref handler) => &handler.parent,
            CommonEventHandler::ErrorEventHandler(ref handler) => &handler.parent,
            CommonEventHandler::BeforeUnloadEventHandler(ref handler) => &handler.parent,
        }
    }
}

#[derive(JSTraceable, Copy, Clone, PartialEq, HeapSizeOf)]
pub enum ListenerPhase {
    Capturing,
    Bubbling,
}

/// https://html.spec.whatwg.org/multipage/#internal-raw-uncompiled-handler
#[derive(JSTraceable, Clone, PartialEq)]
pub struct InternalRawUncompiledHandler {
    source: DOMString,
    url: ServoUrl,
    line: usize,
}

/// A representation of an event handler, either compiled or uncompiled raw source, or null.
#[derive(JSTraceable, PartialEq, Clone)]
pub enum InlineEventListener {
    Uncompiled(InternalRawUncompiledHandler),
    Compiled(CommonEventHandler),
    Null,
}

impl InlineEventListener {
    /// Get a compiled representation of this event handler, compiling it from its
    /// raw source if necessary.
    /// https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    fn get_compiled_handler(&mut self, owner: &EventTarget, ty: &Atom)
                            -> Option<CommonEventHandler> {
        match mem::replace(self, InlineEventListener::Null) {
            InlineEventListener::Null => None,
            InlineEventListener::Uncompiled(handler) => {
                let result = owner.get_compiled_event_handler(handler, ty);
                if let Some(ref compiled) = result {
                    *self = InlineEventListener::Compiled(compiled.clone());
                }
                result
            }
            InlineEventListener::Compiled(handler) => {
                *self = InlineEventListener::Compiled(handler.clone());
                Some(handler)
            }
        }
    }
}

#[derive(JSTraceable, Clone, PartialEq)]
enum EventListenerType {
    Additive(Rc<EventListener>),
    Inline(InlineEventListener),
}

impl HeapSizeOf for EventListenerType {
    fn heap_size_of_children(&self) -> usize {
        // FIXME: Rc<T> isn't HeapSizeOf and we can't ignore it due to #6870 and #6871
        0
    }
}

impl EventListenerType {
    fn get_compiled_listener(&mut self, owner: &EventTarget, ty: &Atom)
                             -> Option<CompiledEventListener> {
        match self {
            &mut EventListenerType::Inline(ref mut inline) =>
                inline.get_compiled_handler(owner, ty)
                      .map(CompiledEventListener::Handler),
            &mut EventListenerType::Additive(ref listener) =>
                Some(CompiledEventListener::Listener(listener.clone())),
        }
    }
}

/// A representation of an EventListener/EventHandler object that has previously
/// been compiled successfully, if applicable.
pub enum CompiledEventListener {
    Listener(Rc<EventListener>),
    Handler(CommonEventHandler),
}

impl CompiledEventListener {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#the-event-handler-processing-algorithm
    pub fn call_or_handle_event<T: Reflectable>(&self,
                                                object: &T,
                                                event: &Event,
                                                exception_handle: ExceptionHandling) {
        // Step 3
        match *self {
            CompiledEventListener::Listener(ref listener) => {
                let _ = listener.HandleEvent_(object, event, exception_handle);
            },
            CompiledEventListener::Handler(ref handler) => {
                match *handler {
                    CommonEventHandler::ErrorEventHandler(ref handler) => {
                        if let Some(event) = event.downcast::<ErrorEvent>() {
                            let cx = object.global().get_cx();
                            rooted!(in(cx) let error = unsafe { event.Error(cx) });
                            let return_value = handler.Call_(object,
                                                             EventOrString::String(event.Message()),
                                                             Some(event.Filename()),
                                                             Some(event.Lineno()),
                                                             Some(event.Colno()),
                                                             Some(error.handle()),
                                                             exception_handle);
                            // Step 4
                            if let Ok(return_value) = return_value {
                                rooted!(in(cx) let return_value = return_value);
                                if return_value.handle().is_boolean() && return_value.handle().to_boolean() == true {
                                    event.upcast::<Event>().PreventDefault();
                                }
                            }
                            return;
                        }

                        let _ = handler.Call_(object, EventOrString::Event(Root::from_ref(event)),
                                              None, None, None, None, exception_handle);
                    }

                    CommonEventHandler::BeforeUnloadEventHandler(ref handler) => {
                        if let Some(event) = event.downcast::<BeforeUnloadEvent>() {
                            let rv = event.ReturnValue();

                            if let Ok(value) = handler.Call_(object,
                                                             event.upcast::<Event>(),
                                                             exception_handle) {
                                match value {
                                    Some(value) => {
                                        if rv.is_empty() {
                                            event.SetReturnValue(value);
                                        }
                                    }
                                    None => {
                                        event.upcast::<Event>().PreventDefault();
                                    }
                                }
                            }
                        }
                    }

                    CommonEventHandler::EventHandler(ref handler) => {
                        if let Ok(value) = handler.Call_(object, event, exception_handle) {
                            let cx = object.global().get_cx();
                            rooted!(in(cx) let value = value);
                            let value = value.handle();

                            //Step 4
                            let should_cancel = match event.type_() {
                                atom!("mouseover") => value.is_boolean() && value.to_boolean() == true,
                                _ => value.is_boolean() && value.to_boolean() == false
                            };
                            if should_cancel {
                                event.PreventDefault();
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(JSTraceable, Clone, PartialEq, HeapSizeOf)]
#[privatize]
/// A listener in a collection of event listeners.
struct EventListenerEntry {
    phase: ListenerPhase,
    listener: EventListenerType
}

#[derive(JSTraceable, HeapSizeOf)]
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
    fn get_inline_listener(&mut self, owner: &EventTarget, ty: &Atom) -> Option<CommonEventHandler> {
        for entry in &mut self.0 {
            if let EventListenerType::Inline(ref mut inline) = entry.listener {
                // Step 1.1-1.8 and Step 2
                return inline.get_compiled_handler(owner, ty);
            }
        }

        // Step 2
        None
    }

    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    fn get_listeners(&mut self, phase: Option<ListenerPhase>, owner: &EventTarget, ty: &Atom)
                     -> Vec<CompiledEventListener> {
        self.0.iter_mut().filter_map(|entry| {
            if phase.is_none() || Some(entry.phase) == phase {
                // Step 1.1-1.8, 2
                entry.listener.get_compiled_listener(owner, ty)
            } else {
                None
            }
        }).collect()
    }
}

#[dom_struct]
pub struct EventTarget {
    reflector_: Reflector,
    handlers: DOMRefCell<HashMap<Atom, EventListeners, BuildHasherDefault<FnvHasher>>>,
}

impl EventTarget {
    pub fn new_inherited() -> EventTarget {
        EventTarget {
            reflector_: Reflector::new(),
            handlers: DOMRefCell::new(Default::default()),
        }
    }

    pub fn get_listeners_for(&self,
                             type_: &Atom,
                             specific_phase: Option<ListenerPhase>)
                             -> Vec<CompiledEventListener> {
        self.handlers.borrow_mut().get_mut(type_).map_or(vec![], |listeners| {
            listeners.get_listeners(specific_phase, self, type_)
        })
    }

    pub fn dispatch_event_with_target(&self,
                                      target: &EventTarget,
                                      event: &Event) -> EventStatus {
        dispatch_event(self, Some(target), event)
    }

    pub fn dispatch_event(&self, event: &Event) -> EventStatus {
        dispatch_event(self, None, event)
    }

    /// https://html.spec.whatwg.org/multipage/#event-handler-attributes:event-handlers-11
    pub fn set_inline_event_listener(&self,
                                     ty: Atom,
                                     listener: Option<InlineEventListener>) {
        let mut handlers = self.handlers.borrow_mut();
        let entries = match handlers.entry(ty) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(EventListeners(vec!())),
        };

        let idx = entries.iter().position(|ref entry| {
            match entry.listener {
                EventListenerType::Inline(_) => true,
                _ => false,
            }
        });

        match idx {
            Some(idx) => {
                entries[idx].listener =
                    EventListenerType::Inline(listener.unwrap_or(InlineEventListener::Null));
            }
            None => {
                if let Some(listener) = listener {
                    entries.push(EventListenerEntry {
                        phase: ListenerPhase::Bubbling,
                        listener: EventListenerType::Inline(listener),
                    });
                }
            }
        }
    }

    fn get_inline_event_listener(&self, ty: &Atom) -> Option<CommonEventHandler> {
        let mut handlers = self.handlers.borrow_mut();
        handlers.get_mut(ty).and_then(|entry| entry.get_inline_listener(self, ty))
    }

    /// Store the raw uncompiled event handler for on-demand compilation later.
    /// https://html.spec.whatwg.org/multipage/#event-handler-attributes:event-handler-content-attributes-3
    pub fn set_event_handler_uncompiled(&self,
                                        url: ServoUrl,
                                        line: usize,
                                        ty: &str,
                                        source: DOMString) {
        let handler = InternalRawUncompiledHandler {
            source: source,
            line: line,
            url: url,
        };
        self.set_inline_event_listener(Atom::from(ty),
                                       Some(InlineEventListener::Uncompiled(handler)));
    }

    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    #[allow(unsafe_code)]
    pub fn get_compiled_event_handler(&self,
                                      handler: InternalRawUncompiledHandler,
                                      ty: &Atom)
                                      -> Option<CommonEventHandler> {
        // Step 1.1
        let element = self.downcast::<Element>();
        let document = match element {
            Some(element) => document_from_node(element),
            None => self.downcast::<Window>().unwrap().Document(),
        };

        // Step 1.2
        if !document.is_scripting_enabled() {
            return None;
        }

        // Step 1.3
        let body: Vec<u16> = handler.source.encode_utf16().collect();

        // TODO step 1.5 (form owner)

        // Step 1.6
        let window = document.window();

        let url_serialized = CString::new(handler.url.to_string()).unwrap();
        let name = CString::new(&**ty).unwrap();

        static mut ARG_NAMES: [*const c_char; 1] = [b"event\0" as *const u8 as *const c_char];
        static mut ERROR_ARG_NAMES: [*const c_char; 5] = [b"event\0" as *const u8 as *const c_char,
                                                          b"source\0" as *const u8 as *const c_char,
                                                          b"lineno\0" as *const u8 as *const c_char,
                                                          b"colno\0" as *const u8 as *const c_char,
                                                          b"error\0" as *const u8 as *const c_char];
        // step 10
        let is_error = ty == &atom!("error") && self.is::<Window>();
        let args = unsafe {
            if is_error {
                &ERROR_ARG_NAMES[..]
            } else {
                &ARG_NAMES[..]
            }
        };

        let cx = window.get_cx();
        let options = CompileOptionsWrapper::new(cx, url_serialized.as_ptr(), handler.line as u32);
        // TODO step 1.10.1-3 (document, form owner, element in scope chain)

        let scopechain = AutoObjectVectorWrapper::new(cx);

        let _ac = JSAutoCompartment::new(cx, window.reflector().get_jsobject().get());
        rooted!(in(cx) let mut handler = ptr::null_mut());
        let rv = unsafe {
            CompileFunction(cx,
                            scopechain.ptr,
                            options.ptr,
                            name.as_ptr(),
                            args.len() as u32,
                            args.as_ptr(),
                            body.as_ptr(),
                            body.len() as size_t,
                            handler.handle_mut())
        };
        if !rv || handler.get().is_null() {
            // Step 1.8.2
            unsafe {
                let _ac = JSAutoCompartment::new(cx, self.reflector().get_jsobject().get());
                // FIXME(#13152): dispatch error event.
                report_pending_exception(cx, false);
            }
            // Step 1.8.1 / 1.8.3
            return None;
        }

        // TODO step 1.11-13
        let funobj = unsafe { JS_GetFunctionObject(handler.get()) };
        assert!(!funobj.is_null());
        // Step 1.14
        if is_error {
            Some(CommonEventHandler::ErrorEventHandler(OnErrorEventHandlerNonNull::new(funobj)))
        } else {
            if ty == &atom!("beforeunload") {
                Some(CommonEventHandler::BeforeUnloadEventHandler(
                        OnBeforeUnloadEventHandlerNonNull::new(funobj)))
            } else {
                Some(CommonEventHandler::EventHandler(EventHandlerNonNull::new(funobj)))
            }
        }
    }

    pub fn set_event_handler_common<T: CallbackContainer>(
        &self, ty: &str, listener: Option<Rc<T>>)
    {
        let event_listener = listener.map(|listener|
                                          InlineEventListener::Compiled(
                                              CommonEventHandler::EventHandler(
                                                  EventHandlerNonNull::new(listener.callback()))));
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    pub fn set_error_event_handler<T: CallbackContainer>(
        &self, ty: &str, listener: Option<Rc<T>>)
    {
        let event_listener = listener.map(|listener|
                                          InlineEventListener::Compiled(
                                              CommonEventHandler::ErrorEventHandler(
                                                  OnErrorEventHandlerNonNull::new(listener.callback()))));
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    pub fn set_beforeunload_event_handler<T: CallbackContainer>(&self, ty: &str,
            listener: Option<Rc<T>>) {
        let event_listener = listener.map(|listener|
            InlineEventListener::Compiled(
                CommonEventHandler::BeforeUnloadEventHandler(
                    OnBeforeUnloadEventHandlerNonNull::new(listener.callback())))
        );
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    pub fn get_event_handler_common<T: CallbackContainer>(&self, ty: &str) -> Option<Rc<T>> {
        let listener = self.get_inline_event_listener(&Atom::from(ty));
        listener.map(|listener| CallbackContainer::new(listener.parent().callback()))
    }

    pub fn has_handlers(&self) -> bool {
        !self.handlers.borrow().is_empty()
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub fn fire_event(&self, name: Atom) -> Root<Event> {
        self.fire_event_with_params(name,
                                    EventBubbles::DoesNotBubble,
                                    EventCancelable::NotCancelable)
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub fn fire_bubbling_event(&self, name: Atom) -> Root<Event> {
        self.fire_event_with_params(name,
                                    EventBubbles::Bubbles,
                                    EventCancelable::NotCancelable)
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub fn fire_cancelable_event(&self, name: Atom) -> Root<Event> {
        self.fire_event_with_params(name,
                                    EventBubbles::DoesNotBubble,
                                    EventCancelable::Cancelable)
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub fn fire_bubbling_cancelable_event(&self, name: Atom) -> Root<Event> {
        self.fire_event_with_params(name,
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable)
    }

    // https://dom.spec.whatwg.org/#concept-event-fire
    pub fn fire_event_with_params(&self,
                                  name: Atom,
                                  bubbles: EventBubbles,
                                  cancelable: EventCancelable)
                                  -> Root<Event> {
        let event = Event::new(&self.global(), name, bubbles, cancelable);
        event.fire(self);
        event
    }
}

impl EventTargetMethods for EventTarget {
    // https://dom.spec.whatwg.org/#dom-eventtarget-addeventlistener
    fn AddEventListener(&self,
                        ty: DOMString,
                        listener: Option<Rc<EventListener>>,
                        capture: bool) {
        let listener = match listener {
            Some(l) => l,
            None => return,
        };
        let mut handlers = self.handlers.borrow_mut();
        let entry = match handlers.entry(Atom::from(ty)) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(EventListeners(vec!())),
        };

        let phase = if capture { ListenerPhase::Capturing } else { ListenerPhase::Bubbling };
        let new_entry = EventListenerEntry {
            phase: phase,
            listener: EventListenerType::Additive(listener)
        };
        if !entry.contains(&new_entry) {
            entry.push(new_entry);
        }
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-removeeventlistener
    fn RemoveEventListener(&self,
                           ty: DOMString,
                           listener: Option<Rc<EventListener>>,
                           capture: bool) {
        let ref listener = match listener {
            Some(l) => l,
            None => return,
        };
        let mut handlers = self.handlers.borrow_mut();
        let entry = handlers.get_mut(&Atom::from(ty));
        for entry in entry {
            let phase = if capture { ListenerPhase::Capturing } else { ListenerPhase::Bubbling };
            let old_entry = EventListenerEntry {
                phase: phase,
                listener: EventListenerType::Additive(listener.clone())
            };
            if let Some(position) = entry.iter().position(|e| *e == old_entry) {
                entry.remove(position);
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-dispatchevent
    fn DispatchEvent(&self, event: &Event) -> Fallible<bool> {
        if event.dispatching() || !event.initialized() {
            return Err(Error::InvalidState);
        }
        event.set_trusted(false);
        Ok(match self.dispatch_event(event) {
            EventStatus::Canceled => false,
            EventStatus::NotCanceled => true
        })
    }
}

impl VirtualMethods for EventTarget {
    fn super_type(&self) -> Option<&VirtualMethods> {
        None
    }
}
