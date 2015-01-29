/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::CallbackContainer;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::error::{Fallible, report_pending_exception};
use dom::bindings::error::Error::InvalidState;
use dom::bindings::js::JSRef;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::event::Event;
use dom::eventdispatcher::dispatch_event;
use dom::node::NodeTypeId;
use dom::workerglobalscope::WorkerGlobalScopeTypeId;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTargetTypeId;
use dom::virtualmethods::VirtualMethods;
use js::jsapi::{JS_CompileUCFunction, JS_GetFunctionObject, JS_CloneFunctionObject};
use js::jsapi::{JSContext, JSObject};
use util::fnv::FnvHasher;
use util::str::DOMString;

use libc::{c_char, size_t};
use std::borrow::ToOwned;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_state::DefaultState;
use std::default::Default;
use std::ffi::CString;
use std::ptr;
use url::Url;

use std::collections::HashMap;

#[derive(Copy, PartialEq)]
#[jstraceable]
pub enum ListenerPhase {
    Capturing,
    Bubbling,
}

#[derive(Copy, PartialEq)]
#[jstraceable]
pub enum EventTargetTypeId {
    Node(NodeTypeId),
    WebSocket,
    Window,
    Worker,
    WorkerGlobalScope(WorkerGlobalScopeTypeId),
    XMLHttpRequestEventTarget(XMLHttpRequestEventTargetTypeId)
}

#[derive(Copy, PartialEq)]
#[jstraceable]
pub enum EventListenerType {
    Additive(EventListener),
    Inline(EventListener),
}

impl EventListenerType {
    fn get_listener(&self) -> EventListener {
        match *self {
            EventListenerType::Additive(listener) |
            EventListenerType::Inline(listener) => listener
        }
    }
}

#[derive(Copy, PartialEq)]
#[jstraceable]
#[privatize]
pub struct EventListenerEntry {
    phase: ListenerPhase,
    listener: EventListenerType
}

#[dom_struct]
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

    pub fn get_listeners(&self, type_: &str) -> Option<Vec<EventListener>> {
        self.handlers.borrow().get(type_).map(|listeners| {
            listeners.iter().map(|entry| entry.listener.get_listener()).collect()
        })
    }

    pub fn get_listeners_for(&self, type_: &str, desired_phase: ListenerPhase)
        -> Option<Vec<EventListener>> {
        self.handlers.borrow().get(type_).map(|listeners| {
            let filtered = listeners.iter().filter(|entry| entry.phase == desired_phase);
            filtered.map(|entry| entry.listener.get_listener()).collect()
        })
    }

    #[inline]
    pub fn type_id<'a>(&'a self) -> &'a EventTargetTypeId {
        &self.type_id
    }
}

pub trait EventTargetHelpers {
    fn dispatch_event_with_target(self,
                                  target: JSRef<EventTarget>,
                                  event: JSRef<Event>) -> bool;
    fn dispatch_event(self, event: JSRef<Event>) -> bool;
    fn set_inline_event_listener(self,
                                 ty: DOMString,
                                 listener: Option<EventListener>);
    fn get_inline_event_listener(self, ty: DOMString) -> Option<EventListener>;
    fn set_event_handler_uncompiled(self,
                                    cx: *mut JSContext,
                                    url: Url,
                                    scope: *mut JSObject,
                                    ty: &str,
                                    source: DOMString);
    fn set_event_handler_common<T: CallbackContainer>(self, ty: &str,
                                                      listener: Option<T>);
    fn get_event_handler_common<T: CallbackContainer>(self, ty: &str) -> Option<T>;

    fn has_handlers(self) -> bool;
}

impl<'a> EventTargetHelpers for JSRef<'a, EventTarget> {
    fn dispatch_event_with_target(self,
                                  target: JSRef<EventTarget>,
                                  event: JSRef<Event>) -> bool {
        dispatch_event(self, Some(target), event)
    }

    fn dispatch_event(self, event: JSRef<Event>) -> bool {
        dispatch_event(self, None, event)
    }

    fn set_inline_event_listener(self,
                                 ty: DOMString,
                                 listener: Option<EventListener>) {
        let mut handlers = self.handlers.borrow_mut();
        let entries = match handlers.entry(ty) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(vec!()),
        };

        let idx = entries.iter().position(|&entry| {
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

    fn get_inline_event_listener(self, ty: DOMString) -> Option<EventListener> {
        let handlers = self.handlers.borrow();
        let entries = handlers.get(&ty);
        entries.and_then(|entries| entries.iter().find(|entry| {
            match entry.listener {
                EventListenerType::Inline(_) => true,
                _ => false,
            }
        }).map(|entry| entry.listener.get_listener()))
    }

    #[allow(unsafe_blocks)]
    fn set_event_handler_uncompiled(self,
                                    cx: *mut JSContext,
                                    url: Url,
                                    scope: *mut JSObject,
                                    ty: &str,
                                    source: DOMString) {
        let url = CString::from_slice(url.serialize().as_bytes());
        let name = CString::from_slice(ty.as_bytes());
        let lineno = 0; //XXXjdm need to get a real number here

        let nargs = 1; //XXXjdm not true for onerror
        const ARG_NAME: [c_char; 6] =
            ['e' as c_char, 'v' as c_char, 'e' as c_char, 'n' as c_char, 't' as c_char, 0];
        static mut ARG_NAMES: [*const c_char; 1] = [&ARG_NAME as *const c_char];

        let source: Vec<u16> = source.as_slice().utf16_units().collect();
        let handler = unsafe {
            JS_CompileUCFunction(cx,
                                 ptr::null_mut(),
                                 name.as_ptr(),
                                 nargs,
                                 &ARG_NAMES as *const *const c_char as *mut *const c_char,
                                 source.as_ptr(),
                                 source.len() as size_t,
                                 url.as_ptr(),
                                 lineno)
        };
        if handler.is_null() {
            report_pending_exception(cx, self.reflector().get_jsobject());
            return;
        }

        let funobj = unsafe {
            JS_CloneFunctionObject(cx, JS_GetFunctionObject(handler), scope)
        };
        assert!(!funobj.is_null());
        self.set_event_handler_common(ty, Some(EventHandlerNonNull::new(funobj)));
    }

    fn set_event_handler_common<T: CallbackContainer>(
        self, ty: &str, listener: Option<T>)
    {
        let event_listener = listener.map(|listener|
                                          EventListener::new(listener.callback()));
        self.set_inline_event_listener(ty.to_owned(), event_listener);
    }

    fn get_event_handler_common<T: CallbackContainer>(self, ty: &str) -> Option<T> {
        let listener = self.get_inline_event_listener(ty.to_owned());
        listener.map(|listener| CallbackContainer::new(listener.parent.callback()))
    }

    fn has_handlers(self) -> bool {
        !self.handlers.borrow().is_empty()
    }
}

impl<'a> EventTargetMethods for JSRef<'a, EventTarget> {
    fn AddEventListener(self,
                        ty: DOMString,
                        listener: Option<EventListener>,
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
                if entry.as_slice().position_elem(&new_entry).is_none() {
                    entry.push(new_entry);
                }
            },
            _ => (),
        }
    }

    fn RemoveEventListener(self,
                           ty: DOMString,
                           listener: Option<EventListener>,
                           capture: bool) {
        match listener {
            Some(listener) => {
                let mut handlers = self.handlers.borrow_mut();
                let mut entry = handlers.get_mut(&ty);
                for entry in entry.iter_mut() {
                    let phase = if capture { ListenerPhase::Capturing } else { ListenerPhase::Bubbling };
                    let old_entry = EventListenerEntry {
                        phase: phase,
                        listener: EventListenerType::Additive(listener)
                    };
                    let position = entry.as_slice().position_elem(&old_entry);
                    for &position in position.iter() {
                        entry.remove(position);
                    }
                }
            },
            _ => (),
        }
    }

    fn DispatchEvent(self, event: JSRef<Event>) -> Fallible<bool> {
        if event.dispatching() || !event.initialized() {
            return Err(InvalidState);
        }
        Ok(self.dispatch_event(event))
    }
}

impl<'a> VirtualMethods for JSRef<'a, EventTarget> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        None
    }
}
