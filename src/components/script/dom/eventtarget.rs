/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JSRef;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::error::{Fallible, InvalidState};
use dom::bindings::codegen::BindingDeclarations::EventListenerBinding;
use self::EventListenerBinding::EventListener;
use dom::event::Event;
use dom::eventdispatcher::dispatch_event;
use dom::node::NodeTypeId;
use dom::xmlhttprequest::XMLHttpRequestId;
use dom::virtualmethods::VirtualMethods;
use servo_util::str::DOMString;

use collections::hashmap::HashMap;

#[deriving(Eq,Encodable)]
pub enum ListenerPhase {
    Capturing,
    Bubbling,
}

#[deriving(Eq,Encodable)]
pub enum EventTargetTypeId {
    NodeTargetTypeId(NodeTypeId),
    WindowTypeId,
    XMLHttpRequestTargetTypeId(XMLHttpRequestId)
}

#[deriving(Eq,Encodable)]
pub struct EventListenerEntry {
    pub phase: ListenerPhase,
    pub listener: EventListener
}

#[deriving(Encodable)]
pub struct EventTarget {
    pub type_id: EventTargetTypeId,
    pub reflector_: Reflector,
    pub handlers: HashMap<DOMString, Vec<EventListenerEntry>>,
}

impl EventTarget {
    pub fn new_inherited(type_id: EventTargetTypeId) -> EventTarget {
        EventTarget {
            type_id: type_id,
            reflector_: Reflector::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn get_listeners(&self, type_: &str) -> Option<Vec<EventListener>> {
        self.handlers.find_equiv(&type_).map(|listeners| {
            listeners.iter().map(|entry| entry.listener).collect()
        })
    }

    pub fn get_listeners_for(&self, type_: &str, desired_phase: ListenerPhase)
        -> Option<Vec<EventListener>> {
        self.handlers.find_equiv(&type_).map(|listeners| {
            let filtered = listeners.iter().filter(|entry| entry.phase == desired_phase);
            filtered.map(|entry| entry.listener).collect()
        })
    }
}

pub trait EventTargetHelpers {
    fn dispatch_event_with_target<'a>(&self,
                                      target: Option<JSRef<'a, EventTarget>>,
                                      event: &mut JSRef<Event>) -> Fallible<bool>;
}

impl<'a> EventTargetHelpers for JSRef<'a, EventTarget> {
    fn dispatch_event_with_target<'b>(&self,
                                      target: Option<JSRef<'b, EventTarget>>,
                                      event: &mut JSRef<Event>) -> Fallible<bool> {
        if event.deref().dispatching || !event.deref().initialized {
            return Err(InvalidState);
        }
        Ok(dispatch_event(self, target, event))
    }
}

pub trait EventTargetMethods {
    fn AddEventListener(&mut self,
                        ty: DOMString,
                        listener: Option<EventListener>,
                        capture: bool);
    fn RemoveEventListener(&mut self,
                           ty: DOMString,
                           listener: Option<EventListener>,
                           capture: bool);
    fn DispatchEvent(&self, event: &mut JSRef<Event>) -> Fallible<bool>;
}

impl<'a> EventTargetMethods for JSRef<'a, EventTarget> {
    fn AddEventListener(&mut self,
                        ty: DOMString,
                        listener: Option<EventListener>,
                        capture: bool) {
        for &listener in listener.iter() {
            let entry = self.handlers.find_or_insert_with(ty.clone(), |_| vec!());
            let phase = if capture { Capturing } else { Bubbling };
            let new_entry = EventListenerEntry {
                phase: phase,
                listener: listener
            };
            if entry.as_slice().position_elem(&new_entry).is_none() {
                entry.push(new_entry);
            }
        }
    }

    fn RemoveEventListener(&mut self,
                           ty: DOMString,
                           listener: Option<EventListener>,
                           capture: bool) {
        for &listener in listener.iter() {
            let mut entry = self.handlers.find_mut(&ty);
            for entry in entry.mut_iter() {
                let phase = if capture { Capturing } else { Bubbling };
                let old_entry = EventListenerEntry {
                    phase: phase,
                    listener: listener
                };
                let position = entry.as_slice().position_elem(&old_entry);
                for &position in position.iter() {
                    entry.remove(position);
                }
            }
        }
    }

    fn DispatchEvent(&self, event: &mut JSRef<Event>) -> Fallible<bool> {
        self.dispatch_event_with_target(None, event)
    }
}

impl Reflectable for EventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

impl<'a> VirtualMethods for JSRef<'a, EventTarget> {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:> {
        None
    }
}
