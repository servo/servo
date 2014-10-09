/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::CallbackContainer;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::error::{Fallible, InvalidState, report_pending_exception};
use dom::bindings::js::JSRef;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::event::Event;
use dom::eventdispatcher::dispatch_event;
use dom::node::NodeTypeId;
use dom::workerglobalscope::WorkerGlobalScopeId;
use dom::xmlhttprequest::XMLHttpRequestId;
use dom::virtualmethods::VirtualMethods;
use js::jsapi::{JS_CompileUCFunction, JS_GetFunctionObject, JS_CloneFunctionObject};
use js::jsapi::{JSContext, JSObject};
use servo_util::str::DOMString;
use libc::{c_char, size_t};
use std::cell::RefCell;
use std::ptr;
use url::Url;

use std::collections::hashmap::HashMap;

#[deriving(PartialEq)]
#[jstraceable]
pub enum ListenerPhase {
    Capturing,
    Bubbling,
}

#[deriving(PartialEq)]
#[jstraceable]
pub enum EventTargetTypeId {
    NodeTargetTypeId(NodeTypeId),
    WindowTypeId,
    WorkerTypeId,
    WorkerGlobalScopeTypeId(WorkerGlobalScopeId),
    XMLHttpRequestTargetTypeId(XMLHttpRequestId)
}

#[deriving(PartialEq)]
#[jstraceable]
pub enum EventListenerType {
    Additive(EventListener),
    Inline(EventListener),
}

impl EventListenerType {
    fn get_listener(&self) -> EventListener {
        match *self {
            Additive(listener) | Inline(listener) => listener
        }
    }
}

#[deriving(PartialEq)]
#[jstraceable]
pub struct EventListenerEntry {
    pub phase: ListenerPhase,
    pub listener: EventListenerType
}

#[jstraceable]
#[must_root]
pub struct EventTarget {
    pub type_id: EventTargetTypeId,
    reflector_: Reflector,
    handlers: RefCell<HashMap<DOMString, Vec<EventListenerEntry>>>,
}

impl EventTarget {
    pub fn new_inherited(type_id: EventTargetTypeId) -> EventTarget {
        EventTarget {
            type_id: type_id,
            reflector_: Reflector::new(),
            handlers: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_listeners(&self, type_: &str) -> Option<Vec<EventListener>> {
        self.handlers.borrow().find_equiv(&type_).map(|listeners| {
            listeners.iter().map(|entry| entry.listener.get_listener()).collect()
        })
    }

    pub fn get_listeners_for(&self, type_: &str, desired_phase: ListenerPhase)
        -> Option<Vec<EventListener>> {
        self.handlers.borrow().find_equiv(&type_).map(|listeners| {
            let filtered = listeners.iter().filter(|entry| entry.phase == desired_phase);
            filtered.map(|entry| entry.listener.get_listener()).collect()
        })
    }
}

pub trait EventTargetHelpers {
    fn dispatch_event_with_target(self,
                                  target: Option<JSRef<EventTarget>>,
                                  event: JSRef<Event>) -> Fallible<bool>;
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
                                  target: Option<JSRef<EventTarget>>,
                                  event: JSRef<Event>) -> Fallible<bool> {
        if event.dispatching.get() || !event.initialized.get() {
            return Err(InvalidState);
        }
        Ok(dispatch_event(self, target, event))
    }

    fn set_inline_event_listener(self,
                                 ty: DOMString,
                                 listener: Option<EventListener>) {
        let mut handlers = self.handlers.borrow_mut();
        let entries = handlers.find_or_insert_with(ty, |_| vec!());
        let idx = entries.iter().position(|&entry| {
            match entry.listener {
                Inline(_) => true,
                _ => false,
            }
        });

        match idx {
            Some(idx) => {
                match listener {
                    Some(listener) => entries.get_mut(idx).listener = Inline(listener),
                    None => {
                        entries.remove(idx);
                    }
                }
            }
            None => {
                if listener.is_some() {
                    entries.push(EventListenerEntry {
                        phase: Bubbling,
                        listener: Inline(listener.unwrap()),
                    });
                }
            }
        }
    }

    fn get_inline_event_listener(self, ty: DOMString) -> Option<EventListener> {
        let handlers = self.handlers.borrow();
        let entries = handlers.find(&ty);
        entries.and_then(|entries| entries.iter().find(|entry| {
            match entry.listener {
                Inline(_) => true,
                _ => false,
            }
        }).map(|entry| entry.listener.get_listener()))
    }

    fn set_event_handler_uncompiled(self,
                                    cx: *mut JSContext,
                                    url: Url,
                                    scope: *mut JSObject,
                                    ty: &str,
                                    source: DOMString) {
        let url = url.serialize().to_c_str();
        let name = ty.to_c_str();
        let lineno = 0; //XXXjdm need to get a real number here

        let nargs = 1; //XXXjdm not true for onerror
        static arg_name: [c_char, ..6] =
            ['e' as c_char, 'v' as c_char, 'e' as c_char, 'n' as c_char, 't' as c_char, 0];
        static arg_names: [*const c_char, ..1] = [&arg_name as *const c_char];

        let source: Vec<u16> = source.as_slice().utf16_units().collect();
        let handler = unsafe {
                JS_CompileUCFunction(cx,
                                     ptr::null_mut(),
                                     name.as_ptr(),
                                     nargs,
                                     &arg_names as *const *const i8 as *mut *const i8,
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
        assert!(funobj.is_not_null());
        self.set_event_handler_common(ty, Some(EventHandlerNonNull::new(funobj)));
    }

    fn set_event_handler_common<T: CallbackContainer>(
        self, ty: &str, listener: Option<T>)
    {
        let event_listener = listener.map(|listener|
                                          EventListener::new(listener.callback()));
        self.set_inline_event_listener(ty.to_string(), event_listener);
    }

    fn get_event_handler_common<T: CallbackContainer>(self, ty: &str) -> Option<T> {
        let listener = self.get_inline_event_listener(ty.to_string());
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
                let entry = handlers.find_or_insert_with(ty, |_| vec!());
                let phase = if capture { Capturing } else { Bubbling };
                let new_entry = EventListenerEntry {
                    phase: phase,
                    listener: Additive(listener)
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
                let mut entry = handlers.find_mut(&ty);
                for entry in entry.iter_mut() {
                    let phase = if capture { Capturing } else { Bubbling };
                    let old_entry = EventListenerEntry {
                        phase: phase,
                        listener: Additive(listener)
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
        self.dispatch_event_with_target(None, event)
    }
}

impl Reflectable for EventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

impl<'a> VirtualMethods for JSRef<'a, EventTarget> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        None
    }
}
