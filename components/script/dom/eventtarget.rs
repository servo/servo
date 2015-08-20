/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::{CallbackContainer, ExceptionHandling};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::error::Error::InvalidState;
use dom::bindings::error::{Fallible, report_pending_exception};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::event::{Event, EventHelpers};
use dom::eventdispatcher::dispatch_event;
use dom::node::NodeTypeId;
use dom::virtualmethods::VirtualMethods;
use dom::workerglobalscope::WorkerGlobalScopeTypeId;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTargetTypeId;
use js::jsapi::{CompileFunction, JS_GetFunctionObject};
use js::jsapi::{JSAutoCompartment, JSAutoRequest};
use js::jsapi::{JSContext, RootedFunction, HandleObject};
use js::rust::{AutoObjectVectorWrapper, CompileOptionsWrapper};
use util::mem::HeapSizeOf;
use util::str::DOMString;

use fnv::FnvHasher;
use libc::{c_char, size_t};
use std::borrow::ToOwned;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_state::DefaultState;
use std::default::Default;
use std::ffi::CString;
use std::intrinsics;
use std::ptr;
use std::rc::Rc;
use url::Url;

use std::collections::HashMap;

pub type EventHandler = EventHandlerNonNull;

#[derive(JSTraceable, Copy, Clone, PartialEq, HeapSizeOf)]
pub enum ListenerPhase {
    Capturing,
    Bubbling,
}

#[derive(JSTraceable, Copy, Clone)]
#[derive(HeapSizeOf)]
pub enum EventTargetTypeId {
    Node(NodeTypeId),
    WebSocket,
    Window,
    Worker,
    FileReader,
    WorkerGlobalScope(WorkerGlobalScopeTypeId),
    XMLHttpRequestEventTarget(XMLHttpRequestEventTargetTypeId)
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
    Inline(Rc<EventHandler>),
}

impl HeapSizeOf for EventListenerType {
    fn heap_size_of_children(&self) -> usize {
        // FIXME: Rc<T> isn't HeapSizeOf and we can't ignore it due to #6870 and #6871
        0
    }
}

impl EventListenerType {
    pub fn call_or_handle_event<T: Reflectable>(&self,
                                                object: &T,
                                                event: &Event,
                                                exception_handle: ExceptionHandling) {
        match *self {
            EventListenerType::Additive(ref listener) => {
                let _ = listener.HandleEvent_(object, event, exception_handle);
            },
            EventListenerType::Inline(ref handler) => {
                let _ = handler.Call_(object, event, exception_handle);
            },
        }
    }
}

#[derive(JSTraceable, Clone, PartialEq, HeapSizeOf)]
#[privatize]
pub struct EventListenerEntry {
    phase: ListenerPhase,
    listener: EventListenerType
}

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct EventTarget {
    reflector_: Reflector,
    type_id: EventTargetTypeId,
    handlers: DOMRefCell<HashMap<DOMString, Vec<EventListenerEntry>, DefaultState<FnvHasher>>>,
}

impl EventTarget {
    pub fn new_inherited(type_id: EventTargetTypeId) -> EventTarget {
        EventTarget {
            reflector_: Reflector::new(),
            type_id: type_id,
            handlers: DOMRefCell::new(Default::default()),
        }
    }

    pub fn get_listeners(&self, type_: &str) -> Option<Vec<EventListenerType>> {
        self.handlers.borrow().get(type_).map(|listeners| {
            listeners.iter().map(|entry| entry.listener.clone()).collect()
        })
    }

    pub fn get_listeners_for(&self, type_: &str, desired_phase: ListenerPhase)
        -> Option<Vec<EventListenerType>> {
        self.handlers.borrow().get(type_).map(|listeners| {
            let filtered = listeners.iter().filter(|entry| entry.phase == desired_phase);
            filtered.map(|entry| entry.listener.clone()).collect()
        })
    }

    #[inline]
    pub fn type_id<'a>(&'a self) -> &'a EventTargetTypeId {
        &self.type_id
    }
}

pub trait EventTargetHelpers {
    fn dispatch_event_with_target(self,
                                  target: &EventTarget,
                                  event: &Event) -> bool;
    fn dispatch_event(self, event: &Event) -> bool;
    fn set_inline_event_listener(self,
                                 ty: DOMString,
                                 listener: Option<Rc<EventHandler>>);
    fn get_inline_event_listener(self, ty: DOMString) -> Option<Rc<EventHandler>>;
    fn set_event_handler_uncompiled(self,
                                    cx: *mut JSContext,
                                    url: Url,
                                    scope: HandleObject,
                                    ty: &str,
                                    source: DOMString);
    fn set_event_handler_common<T: CallbackContainer>(self, ty: &str,
                                                      listener: Option<Rc<T>>);
    fn get_event_handler_common<T: CallbackContainer>(self, ty: &str) -> Option<Rc<T>>;

    fn has_handlers(self) -> bool;
}

impl<'a> EventTargetHelpers for &'a EventTarget {
    fn dispatch_event_with_target(self,
                                  target: &EventTarget,
                                  event: &Event) -> bool {
        dispatch_event(self, Some(target), event)
    }

    fn dispatch_event(self, event: &Event) -> bool {
        dispatch_event(self, None, event)
    }

    fn set_inline_event_listener(self,
                                 ty: DOMString,
                                 listener: Option<Rc<EventHandler>>) {
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

    fn get_inline_event_listener(self, ty: DOMString) -> Option<Rc<EventHandler>> {
        let handlers = self.handlers.borrow();
        let entries = handlers.get(&ty);
        entries.and_then(|entries| entries.iter().filter_map(|entry| {
            match entry.listener {
                EventListenerType::Inline(ref handler) => Some(handler.clone()),
                _ => None,
            }
        }).next())
    }

    #[allow(unsafe_code)]
    fn set_event_handler_uncompiled(self,
                                    cx: *mut JSContext,
                                    url: Url,
                                    scope: HandleObject,
                                    ty: &str,
                                    source: DOMString) {
        let url = CString::new(url.serialize()).unwrap();
        let name = CString::new(ty).unwrap();
        let lineno = 0; //XXXjdm need to get a real number here

        let nargs = 1; //XXXjdm not true for onerror
        static mut ARG_NAMES: [*const c_char; 1] = [b"event\0" as *const u8 as *const c_char];

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
                            nargs,
                            ARG_NAMES.as_mut_ptr(),
                            source.as_ptr() as *const i16,
                            source.len() as size_t,
                            handler.handle_mut())
        };
        if rv == 0 || handler.ptr.is_null() {
            report_pending_exception(cx, self.reflector().get_jsobject().get());
            return;
        }

        let funobj = unsafe { JS_GetFunctionObject(handler.ptr) };
        assert!(!funobj.is_null());
        self.set_event_handler_common(ty, Some(EventHandlerNonNull::new(funobj)));
    }

    fn set_event_handler_common<T: CallbackContainer>(
        self, ty: &str, listener: Option<Rc<T>>)
    {
        let event_listener = listener.map(|listener|
                                          EventHandlerNonNull::new(listener.callback()));
        self.set_inline_event_listener(ty.to_owned(), event_listener);
    }

    fn get_event_handler_common<T: CallbackContainer>(self, ty: &str) -> Option<Rc<T>> {
        let listener = self.get_inline_event_listener(ty.to_owned());
        listener.map(|listener| CallbackContainer::new(listener.parent.callback()))
    }

    fn has_handlers(self) -> bool {
        !self.handlers.borrow().is_empty()
    }
}

impl<'a> EventTargetMethods for &'a EventTarget {
    // https://dom.spec.whatwg.org/#dom-eventtarget-addeventlistener
    fn AddEventListener(self,
                        ty: DOMString,
                        listener: Option<Rc<EventListener>>,
                        capture: bool) {
        match listener {
            Some(listener) => {
                let mut handlers = self.handlers.borrow_mut();
                let entry = match handlers.entry(ty) {
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
    fn RemoveEventListener(self,
                           ty: DOMString,
                           listener: Option<Rc<EventListener>>,
                           capture: bool) {
        match listener {
            Some(ref listener) => {
                let mut handlers = self.handlers.borrow_mut();
                let entry = handlers.get_mut(&ty);
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
    fn DispatchEvent(self, event: &Event) -> Fallible<bool> {
        if event.dispatching() || !event.initialized() {
            return Err(InvalidState);
        }
        event.set_trusted(false);
        Ok(self.dispatch_event(event))
    }
}

impl<'a> VirtualMethods for &'a EventTarget {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        None
    }
}
