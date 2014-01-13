/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector, DOMString, Fallible};
use dom::bindings::utils::{InvalidState};
use dom::bindings::codegen::EventListenerBinding::EventListener;
use dom::document::AbstractDocument;
use dom::event::AbstractEvent;
use dom::eventdispatcher::dispatch_event;
use dom::node::AbstractNode;
use dom::window::Window;

use std::cast;
use std::hashmap::HashMap;
use std::unstable::raw::Box;

#[deriving(Eq)]
pub enum ListenerPhase {
    Capturing,
    Bubbling,
}

#[deriving(Eq)]
pub enum EventTargetTypeId {
    WindowTypeId,
    NodeTypeId
}

#[deriving(Eq)]
struct EventListenerEntry {
    phase: ListenerPhase,
    listener: EventListener
}

pub struct EventTarget {
    type_id: EventTargetTypeId,
    reflector_: Reflector,
    handlers: HashMap<~str, ~[EventListenerEntry]>,
}

pub struct AbstractEventTarget {
    eventtarget: *mut Box<EventTarget>
}

impl AbstractEventTarget {
    pub fn from_box<T>(box_: *mut Box<T>) -> AbstractEventTarget {
        AbstractEventTarget {
            eventtarget: box_ as *mut Box<EventTarget>
        }
    }

    pub fn from_node(node: AbstractNode) -> AbstractEventTarget {
        unsafe {
            cast::transmute(node)
        }
    }

    pub fn from_window(window: @mut Window) -> AbstractEventTarget {
        AbstractEventTarget {
            eventtarget: unsafe { cast::transmute(window) }
        }
    }

    pub fn from_document(document: AbstractDocument) -> AbstractEventTarget {
        unsafe {
            cast::transmute(document)
        }
    }

    pub fn type_id(&self) -> EventTargetTypeId {
        self.eventtarget().type_id
    }

    pub fn is_window(&self) -> bool {
        self.type_id() == WindowTypeId
    }

    pub fn is_node(&self) -> bool {
        self.type_id() == NodeTypeId
    }

    //
    // Downcasting borrows
    //

    fn transmute<'a, T>(&'a self) -> &'a T {
        unsafe {
            let box_: *Box<T> = self.eventtarget as *Box<T>;
            &(*box_).data
        }
    }

    fn transmute_mut<'a, T>(&'a mut self) -> &'a mut T {
        unsafe {
            let box_: *mut Box<T> = self.eventtarget as *mut Box<T>;
            &mut (*box_).data
        }
    }

    pub fn eventtarget<'a>(&'a self) -> &'a EventTarget {
        self.transmute()
    }

    pub fn mut_eventtarget<'a>(&'a mut self) -> &'a mut EventTarget {
        self.transmute_mut()
    }
}

impl Reflectable for AbstractEventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.mut_eventtarget().mut_reflector()
    }
}

impl EventTarget {
    pub fn new_inherited(type_id: EventTargetTypeId) -> EventTarget {
        EventTarget {
            type_id: type_id,
            reflector_: Reflector::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn get_listeners(&self, type_: &str) -> Option<~[EventListener]> {
        self.handlers.find_equiv(&type_).map(|listeners| {
            listeners.iter().map(|entry| entry.listener).collect()
        })
    }

    pub fn get_listeners_for(&self, type_: &str, desired_phase: ListenerPhase)
        -> Option<~[EventListener]> {
        self.handlers.find_equiv(&type_).map(|listeners| {
            let filtered = listeners.iter().filter(|entry| entry.phase == desired_phase);
            filtered.map(|entry| entry.listener).collect()
        })
    }

    pub fn AddEventListener(&mut self,
                            ty: DOMString,
                            listener: Option<EventListener>,
                            capture: bool) {
        for &listener in listener.iter() {
            let entry = self.handlers.find_or_insert_with(ty.clone(), |_| ~[]);
            let phase = if capture { Capturing } else { Bubbling };
            let new_entry = EventListenerEntry {
                phase: phase,
                listener: listener
            };
            if entry.position_elem(&new_entry).is_none() {
                entry.push(new_entry);
            }
        }
    }

    pub fn RemoveEventListener(&mut self,
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
                let position = entry.position_elem(&old_entry);
                for &position in position.iter() {
                    entry.remove(position);
                }
            }
        }
    }

    pub fn DispatchEvent(&self, abstract_self: AbstractEventTarget, event: AbstractEvent) -> Fallible<bool> {
        self.dispatch_event_with_target(abstract_self, None, event)
    }

    pub fn dispatch_event_with_target(&self,
                                      abstract_self: AbstractEventTarget,
                                      abstract_target: Option<AbstractEventTarget>,
                                      event: AbstractEvent) -> Fallible<bool> {
        if event.event().dispatching || !event.event().initialized {
            return Err(InvalidState);
        }
        Ok(dispatch_event(abstract_self, abstract_target, event))
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
