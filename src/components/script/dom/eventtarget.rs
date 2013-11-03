/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector, DOMString, Fallible};
use dom::bindings::utils::{null_str_as_word_null, InvalidState};
use dom::bindings::codegen::EventListenerBinding::EventListener;
use dom::event::AbstractEvent;
use dom::eventdispatcher::dispatch_event;
use dom::node::{AbstractNode, ScriptView};
use script_task::page_from_context;

use js::jsapi::JSContext;

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
    pub fn from_box<T>(box: *mut Box<T>) -> AbstractEventTarget {
        AbstractEventTarget {
            eventtarget: box as *mut Box<EventTarget>
        }
    }

    pub fn from_node(node: AbstractNode<ScriptView>) -> AbstractEventTarget {
        unsafe {
            cast::transmute(node)
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
            let box: *Box<T> = self.eventtarget as *Box<T>;
            &(*box).data
        }
    }

    fn transmute_mut<'a, T>(&'a mut self) -> &'a mut T {
        unsafe {
            let box: *mut Box<T> = self.eventtarget as *mut Box<T>;
            &mut (*box).data
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

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        self.eventtarget().GetParentObject(cx)
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

    pub fn get_listeners(&self, type_: ~str) -> Option<~[EventListener]> {
        do self.handlers.find_equiv(&type_).map |listeners| {
            listeners.iter().map(|entry| entry.listener).collect()
        }
    }

    pub fn get_listeners_for(&self, type_: ~str, desired_phase: ListenerPhase)
        -> Option<~[EventListener]> {
        do self.handlers.find_equiv(&type_).map |listeners| {
            let filtered = listeners.iter().filter(|entry| entry.phase == desired_phase);
            filtered.map(|entry| entry.listener).collect()
        }
    }

    pub fn AddEventListener(&mut self,
                            ty: &DOMString,
                            listener: Option<EventListener>,
                            capture: bool) {
        for &listener in listener.iter() {
            let entry = self.handlers.find_or_insert_with(null_str_as_word_null(ty), |_| ~[]);
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
                               ty: &DOMString,
                               listener: Option<EventListener>,
                               capture: bool) {
        for &listener in listener.iter() {
            let mut entry = self.handlers.find_mut(&null_str_as_word_null(ty));
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
        if event.event().dispatching || !event.event().initialized {
            return Err(InvalidState);
        }
        Ok(dispatch_event(abstract_self, event))
    }
}

impl Reflectable for EventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        // TODO(tkuehn): This only handles top-level pages. Needs to handle subframes.
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
