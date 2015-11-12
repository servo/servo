/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::{CallbackContainer, ExceptionHandling, CallbackFunction};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::UnionTypes::EventOrString;
use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::global::global_root_from_reflector;
use dom::bindings::inheritance::{Castable, EventTargetTypeId};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::element::Element;
use dom::errorevent::ErrorEvent;
use dom::event::Event;
use dom::eventdispatcher::dispatch_event;
use dom::node::document_from_node;
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use fnv::FnvHasher;
use js::jsapi::{CompileFunction, JS_GetFunctionObject, RootedValue, RootedFunction};
use js::jsapi::{JSAutoCompartment, JSAutoRequest};
use js::rust::{AutoObjectVectorWrapper, CompileOptionsWrapper};
use libc::{c_char, size_t};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_state::DefaultState;
use std::default::Default;
use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::{intrinsics, ptr};
use string_cache::Atom;
use url::Url;
use util::mem::HeapSizeOf;
use util::str::DOMString;

#[derive(PartialEq, Clone, JSTraceable)]
pub enum CommonEventHandler {
    EventHandler(Rc<EventHandlerNonNull>),
    ErrorEventHandler(Rc<OnErrorEventHandlerNonNull>),
}

impl CommonEventHandler {
    fn parent(&self) -> &CallbackFunction {
        match *self {
            CommonEventHandler::EventHandler(ref handler) => &handler.parent,
            CommonEventHandler::ErrorEventHandler(ref handler) => &handler.parent,
        }
    }
}

#[derive(JSTraceable, Copy, Clone, PartialEq, HeapSizeOf)]
pub enum ListenerPhase {
    Capturing,
    Bubbling,
}

impl PartialEq for EventTargetTypeId {
    #[inline]
    fn eq(&self, other: &EventTargetTypeId) -> bool {
        match (*self, *other) {
            (EventTargetTypeId::Node(this_type), EventTargetTypeId::Node(other_type)) => {
                this_type == other_type
            }
            _ => self.eq_slow(other)
        }
    }
}

impl EventTargetTypeId {
    #[allow(unsafe_code)]
    fn eq_slow(&self, other: &EventTargetTypeId) -> bool {
        match (*self, *other) {
            (EventTargetTypeId::Node(this_type), EventTargetTypeId::Node(other_type)) => {
                this_type == other_type
            }
            (EventTargetTypeId::WorkerGlobalScope(this_type),
             EventTargetTypeId::WorkerGlobalScope(other_type)) => {
                this_type == other_type
            }
            (EventTargetTypeId::XMLHttpRequestEventTarget(this_type),
             EventTargetTypeId::XMLHttpRequestEventTarget(other_type)) => {
                this_type == other_type
            }
            (_, _) => {
                unsafe {
                    intrinsics::discriminant_value(self) == intrinsics::discriminant_value(other)
                }
            }
        }
    }
}

/// A representation of an event handler, either compiled or uncompiled raw source.
#[derive(JSTraceable, PartialEq, Clone)]
pub enum InlineEventListener {
    Uncompiled(Option<(DOMString, Url, usize)>),
    Compiled(CommonEventHandler),
}

impl InlineEventListener {
    /// Get a compiled representation of this event handler, compiling it from its
    /// raw source if necessary.
    fn get_compiled_handler(&mut self, owner: &EventTarget, ty: &Atom)
                            -> Option<CommonEventHandler> {
        match self {
            &mut InlineEventListener::Uncompiled(ref mut inner) => {
                let (source, url, line) = inner.take().unwrap();
                owner.get_compiled_event_handler(url, line, ty, source)
            }
            &mut InlineEventListener::Compiled(ref handler) => Some(handler.clone()),
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
                      .map(|h| CompiledEventListener::Handler(h)),
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
                            let global = global_root_from_reflector(object);
                            let cx = global.r().get_cx();
                            let error = RootedValue::new(cx, event.Error(cx));
                            let _ = handler.Call_(object,
                                                  EventOrString::eString(event.Message()),
                                                  Some(event.Filename()),
                                                  Some(event.Lineno()),
                                                  Some(event.Colno()),
                                                  Some(error.handle()),
                                                  exception_handle);
                            return;
                        }

                        let _ = handler.Call_(object, EventOrString::eEvent(Root::from_ref(event)),
                                              None, None, None, None, exception_handle);
                    }

                    CommonEventHandler::EventHandler(ref handler) => {
                        let _ = handler.Call_(object, event, exception_handle);
                    }
                }
            },
        }

        // TODO(#8490): step 4 (cancel event based on return value)
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
/// A mix of potentially uncompiled and compiled event listeners.
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
        let mut to_remove = None;

        for (idx, entry) in self.0.iter_mut().enumerate() {
            if let EventListenerType::Inline(ref mut inline) = entry.listener {
                // Step 1.1-1.7
                let result = inline.get_compiled_handler(owner, ty);
                if result.is_some() {
                    // Step 2
                    return result;
                }

                // Step 1.8
                to_remove = Some(idx);
                break;
            }
        }

        // Step 1.8.1
        if let Some(idx) = to_remove {
            self.0.remove(idx);
        }

        // Step 2
        None
    }

    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    fn get_listeners(&mut self, phase: Option<ListenerPhase>, owner: &EventTarget, ty: &Atom)
                     -> Vec<CompiledEventListener> {
        let mut to_remove = vec![];
        let result = self.0.iter_mut().enumerate().filter_map(|(idx, entry)| {
            if phase.is_none() || Some(entry.phase) == phase {
                // Step 1.1-1.7
                if let Some(listener) = entry.listener.get_compiled_listener(owner, ty) {
                    // Step 2
                    Some(listener)
                } else {
                    // Step 1.8
                    to_remove.push(idx);
                    None
                }
            } else {
                None
            }
        }).collect();

        // Step 1.8.1
        for (position, idx) in to_remove.iter().enumerate() {
            self.0.remove(idx - position);
        }

        // Step 2
        result
    }
}

#[dom_struct]
pub struct EventTarget {
    reflector_: Reflector,
    handlers: DOMRefCell<HashMap<Atom, EventListeners, DefaultState<FnvHasher>>>,
}

impl EventTarget {
    pub fn new_inherited() -> EventTarget {
        EventTarget {
            reflector_: Reflector::new(),
            handlers: DOMRefCell::new(Default::default()),
        }
    }

    pub fn get_listeners(&self, type_: &Atom) -> Option<Vec<CompiledEventListener>> {
        self.handlers.borrow_mut().get_mut(type_).map(|listeners| {
            listeners.get_listeners(None, self, type_)
        })
    }

    pub fn get_listeners_for(&self, type_: &Atom, desired_phase: ListenerPhase)
                             -> Option<Vec<CompiledEventListener>> {
        self.handlers.borrow_mut().get_mut(type_).map(|listeners| {
            listeners.get_listeners(Some(desired_phase), self, type_)
        })
    }

    pub fn dispatch_event_with_target(&self,
                                  target: &EventTarget,
                                  event: &Event) -> bool {
        dispatch_event(self, Some(target), event)
    }

    pub fn dispatch_event(&self, event: &Event) -> bool {
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
                match listener {
                    Some(listener) => entries[idx].listener = EventListenerType::Inline(listener),
                    None => {
                        entries.remove(idx);
                    }
                }
            }
            None => {
                if listener.is_some() {
                    entries.push(EventListenerEntry {
                        phase: ListenerPhase::Bubbling,
                        listener: EventListenerType::Inline(listener.unwrap()),
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
                                        url: Url,
                                        line: usize,
                                        ty: &str,
                                        source: DOMString) {
        self.set_inline_event_listener(Atom::from_slice(ty),
                                       Some(InlineEventListener::Uncompiled(
                                           Some((source, url, line)))));
    }

    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    #[allow(unsafe_code)]
    pub fn get_compiled_event_handler(&self,
                                      url: Url,
                                      lineno: usize,
                                      ty: &Atom,
                                      source: DOMString)
                                      -> Option<CommonEventHandler> {
        // Step 1.1
        let element = self.downcast::<Element>();
        let document = match element {
            Some(element) => document_from_node(element),
            None => self.downcast::<Window>().unwrap().Document(),
        };

        // TODO step 1.2 (browsing context/scripting enabled)

        // Step 1.3
        let body: Vec<u16> = source.utf16_units().collect();

        // TODO step 1.5 (form owner)

        // Step 1.6
        let window = document.window();

        let url_serialized = CString::new(url.serialize()).unwrap();
        let name = CString::new(&**ty).unwrap();

        static mut ARG_NAMES: [*const c_char; 1] = [b"event\0" as *const u8 as *const c_char];
        static mut ERROR_ARG_NAMES: [*const c_char; 5] = [b"event\0" as *const u8 as *const c_char,
                                                          b"source\0" as *const u8 as *const c_char,
                                                          b"lineno\0" as *const u8 as *const c_char,
                                                          b"colno\0" as *const u8 as *const c_char,
                                                          b"error\0" as *const u8 as *const c_char];
        // step 10
        let is_error = ty == &Atom::from_slice("error") && self.is::<Window>();
        let args = unsafe {
            if is_error {
                &ERROR_ARG_NAMES[..]
            } else {
                &ARG_NAMES[..]
            }
        };

        let cx = window.get_cx();
        let options = CompileOptionsWrapper::new(cx, url_serialized.as_ptr(), lineno as u32);
        // TODO step 1.10.1-3 (document, form owner, element in scope chain)

        let scopechain = AutoObjectVectorWrapper::new(cx);

        let _ar = JSAutoRequest::new(cx);
        let _ac = JSAutoCompartment::new(cx, window.reflector().get_jsobject().get());
        let mut handler = RootedFunction::new(cx, ptr::null_mut());
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
        if !rv || handler.ptr.is_null() {
            // Step 1.8.2
            report_pending_exception(cx, self.reflector().get_jsobject().get());
            // Step 1.8.1 / 1.8.3
            return None;
        }

        // TODO step 1.11-13
        let funobj = unsafe { JS_GetFunctionObject(handler.ptr) };
        assert!(!funobj.is_null());
        // Step 1.14
        if is_error {
            Some(CommonEventHandler::ErrorEventHandler(OnErrorEventHandlerNonNull::new(funobj)))
        } else {
            Some(CommonEventHandler::EventHandler(EventHandlerNonNull::new(funobj)))
        }
    }

    pub fn set_event_handler_common<T: CallbackContainer>(
        &self, ty: &str, listener: Option<Rc<T>>)
    {
        let event_listener = listener.map(|listener|
                                          InlineEventListener::Compiled(
                                              CommonEventHandler::EventHandler(
                                                  EventHandlerNonNull::new(listener.callback()))));
        self.set_inline_event_listener(Atom::from_slice(ty), event_listener);
    }

    pub fn set_error_event_handler<T: CallbackContainer>(
        &self, ty: &str, listener: Option<Rc<T>>)
    {
        let event_listener = listener.map(|listener|
                                          InlineEventListener::Compiled(
                                              CommonEventHandler::ErrorEventHandler(
                                                  OnErrorEventHandlerNonNull::new(listener.callback()))));
        self.set_inline_event_listener(Atom::from_slice(ty), event_listener);
    }

    pub fn get_event_handler_common<T: CallbackContainer>(&self, ty: &str) -> Option<Rc<T>> {
        let listener = self.get_inline_event_listener(&Atom::from_slice(ty));
        listener.map(|listener| CallbackContainer::new(listener.parent().callback()))
    }

    pub fn has_handlers(&self) -> bool {
        !self.handlers.borrow().is_empty()
    }
}

impl EventTargetMethods for EventTarget {
    // https://dom.spec.whatwg.org/#dom-eventtarget-addeventlistener
    fn AddEventListener(&self,
                        ty: DOMString,
                        listener: Option<Rc<EventListener>>,
                        capture: bool) {
        match listener {
            Some(listener) => {
                let mut handlers = self.handlers.borrow_mut();
                let entry = match handlers.entry(Atom::from_slice(&ty)) {
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
            },
            _ => (),
        }
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-removeeventlistener
    fn RemoveEventListener(&self,
                           ty: DOMString,
                           listener: Option<Rc<EventListener>>,
                           capture: bool) {
        match listener {
            Some(ref listener) => {
                let mut handlers = self.handlers.borrow_mut();
                let entry = handlers.get_mut(&Atom::from_slice(&ty));
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
            },
            _ => (),
        }
    }

    // https://dom.spec.whatwg.org/#dom-eventtarget-dispatchevent
    fn DispatchEvent(&self, event: &Event) -> Fallible<bool> {
        if event.dispatching() || !event.initialized() {
            return Err(Error::InvalidState);
        }
        event.set_trusted(false);
        Ok(self.dispatch_event(event))
    }
}

impl VirtualMethods for EventTarget {
    fn super_type(&self) -> Option<&VirtualMethods> {
        None
    }
}
