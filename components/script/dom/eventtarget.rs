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
use dom::bindings::codegen::UnionTypes::EventOrString;
use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::global::global_root_from_reflector;
use dom::bindings::inheritance::{Castable, EventTargetTypeId};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::errorevent::ErrorEvent;
use dom::event::Event;
use dom::eventdispatcher::dispatch_event;
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use fnv::FnvHasher;
use js::jsapi::{CompileFunction, JS_GetFunctionObject, RootedValue};
use js::jsapi::{HandleObject, JSContext, RootedFunction};
use js::jsapi::{JSAutoCompartment, JSAutoRequest};
use js::rust::{AutoObjectVectorWrapper, CompileOptionsWrapper};
use libc::{c_char, size_t};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_state::DefaultState;
use std::default::Default;
use std::ffi::CString;
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

#[derive(JSTraceable, Clone, PartialEq)]
pub enum EventListenerType {
    Additive(Rc<EventListener>),
    Inline(CommonEventHandler),
}

impl HeapSizeOf for EventListenerType {
    fn heap_size_of_children(&self) -> usize {
        // FIXME: Rc<T> isn't HeapSizeOf and we can't ignore it due to #6870 and #6871
        0
    }
}

impl EventListenerType {
    // https://html.spec.whatwg.org/multipage/#the-event-handler-processing-algorithm
    pub fn call_or_handle_event<T: Reflectable>(&self,
                                                object: &T,
                                                event: &Event,
                                                exception_handle: ExceptionHandling) {
        // Step 3
        match *self {
            EventListenerType::Additive(ref listener) => {
                let _ = listener.HandleEvent_(object, event, exception_handle);
            },
            EventListenerType::Inline(ref handler) => {
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
pub struct EventListenerEntry {
    phase: ListenerPhase,
    listener: EventListenerType
}

#[dom_struct]
pub struct EventTarget {
    reflector_: Reflector,
    handlers: DOMRefCell<HashMap<Atom, Vec<EventListenerEntry>, DefaultState<FnvHasher>>>,
}

impl EventTarget {
    pub fn new_inherited() -> EventTarget {
        EventTarget {
            reflector_: Reflector::new(),
            handlers: DOMRefCell::new(Default::default()),
        }
    }

    pub fn get_listeners(&self, type_: &Atom) -> Option<Vec<EventListenerType>> {
        self.handlers.borrow().get(type_).map(|listeners| {
            listeners.iter().map(|entry| entry.listener.clone()).collect()
        })
    }

    pub fn get_listeners_for(&self, type_: &Atom, desired_phase: ListenerPhase)
        -> Option<Vec<EventListenerType>> {
        self.handlers.borrow().get(type_).map(|listeners| {
            let filtered = listeners.iter().filter(|entry| entry.phase == desired_phase);
            filtered.map(|entry| entry.listener.clone()).collect()
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

    pub fn set_inline_event_listener(&self,
                                     ty: Atom,
                                     listener: Option<CommonEventHandler>) {
        let mut handlers = self.handlers.borrow_mut();
        let entries = match handlers.entry(ty) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(vec!()),
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

    pub fn get_inline_event_listener(&self, ty: &Atom) -> Option<CommonEventHandler> {
        let handlers = self.handlers.borrow();
        let entries = handlers.get(ty);
        entries.and_then(|entries| entries.iter().filter_map(|entry| {
            match entry.listener {
                EventListenerType::Inline(ref handler) => Some(handler.clone()),
                _ => None,
            }
        }).next())
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#getting-the-current-value-of-the-event-handler
    pub fn set_event_handler_uncompiled(&self,
                                    cx: *mut JSContext,
                                    url: Url,
                                    scope: HandleObject,
                                    ty: &str,
                                    source: DOMString) {
        let url = CString::new(url.serialize()).unwrap();
        let name = CString::new(ty).unwrap();
        let lineno = 0; //XXXjdm need to get a real number here

        static mut ARG_NAMES: [*const c_char; 1] = [b"event\0" as *const u8 as *const c_char];
        static mut ERROR_ARG_NAMES: [*const c_char; 5] = [b"event\0" as *const u8 as *const c_char,
                                                          b"source\0" as *const u8 as *const c_char,
                                                          b"lineno\0" as *const u8 as *const c_char,
                                                          b"colno\0" as *const u8 as *const c_char,
                                                          b"error\0" as *const u8 as *const c_char];
        // step 10
        let is_error = ty == "error" && self.is::<Window>();
        let args = unsafe {
            if is_error {
                &ERROR_ARG_NAMES[..]
            } else {
                &ARG_NAMES[..]
            }
        };

        let source: Vec<u16> = source.utf16_units().collect();
        let options = CompileOptionsWrapper::new(cx, url.as_ptr(), lineno);
        let scopechain = AutoObjectVectorWrapper::new(cx);

        let _ar = JSAutoRequest::new(cx);
        let _ac = JSAutoCompartment::new(cx, scope.get());
        let mut handler = RootedFunction::new(cx, ptr::null_mut());
        let rv = unsafe {
            CompileFunction(cx,
                            scopechain.ptr,
                            options.ptr,
                            name.as_ptr(),
                            args.len() as u32,
                            args.as_ptr(),
                            source.as_ptr(),
                            source.len() as size_t,
                            handler.handle_mut())
        };
        if !rv || handler.ptr.is_null() {
            report_pending_exception(cx, self.reflector().get_jsobject().get());
            return;
        }

        let funobj = unsafe { JS_GetFunctionObject(handler.ptr) };
        assert!(!funobj.is_null());
        if is_error {
            self.set_error_event_handler(ty, Some(OnErrorEventHandlerNonNull::new(funobj)));
        } else {
            self.set_event_handler_common(ty, Some(EventHandlerNonNull::new(funobj)));
        }
    }

    pub fn set_event_handler_common<T: CallbackContainer>(
        &self, ty: &str, listener: Option<Rc<T>>)
    {
        let event_listener = listener.map(|listener|
                                          CommonEventHandler::EventHandler(
                                              EventHandlerNonNull::new(listener.callback())));
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    pub fn set_error_event_handler<T: CallbackContainer>(
        &self, ty: &str, listener: Option<Rc<T>>)
    {
        let event_listener = listener.map(|listener|
                                          CommonEventHandler::ErrorEventHandler(
                                              OnErrorEventHandlerNonNull::new(listener.callback())));
        self.set_inline_event_listener(Atom::from(ty), event_listener);
    }

    pub fn get_event_handler_common<T: CallbackContainer>(&self, ty: &str) -> Option<Rc<T>> {
        let listener = self.get_inline_event_listener(&Atom::from(ty));
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
                let entry = match handlers.entry(Atom::from(&*ty)) {
                    Occupied(entry) => entry.into_mut(),
                    Vacant(entry) => entry.insert(vec!()),
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
                let entry = handlers.get_mut(&Atom::from(&*ty));
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
