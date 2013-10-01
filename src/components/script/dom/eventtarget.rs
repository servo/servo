/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::eReportExceptions;
use dom::bindings::codegen::EventTargetBinding;
use dom::bindings::utils::{Reflectable, Reflector, DOMString, Fallible, DerivedWrapper};
use dom::bindings::utils::null_str_as_word_null;
use dom::bindings::codegen::EventListenerBinding::EventListener;
use dom::event::AbstractEvent;
use dom::node::{AbstractNode, ScriptView};
use script_task::page_from_context;

use js::jsapi::{JSObject, JSContext, JSVal};
use js::glue::RUST_OBJECT_TO_JSVAL;

use std::cast;
use std::hashmap::HashMap;
use std::unstable::raw::Box;

pub struct EventTarget {
    reflector_: Reflector,
    capturing_handlers: HashMap<~str, ~[EventListener]>,
    bubbling_handlers: HashMap<~str, ~[EventListener]>
}

pub struct AbstractEventTarget {
    eventtarget: *mut Box<EventTarget>
}

impl AbstractEventTarget {
    pub fn from_box(box: *mut Box<EventTarget>) -> AbstractEventTarget {
        AbstractEventTarget {
            eventtarget: box
        }
    }

    pub fn from_node(node: AbstractNode<ScriptView>) -> AbstractEventTarget {
        unsafe {
            cast::transmute(node)
        }
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

    fn eventtarget<'a>(&'a self) -> &'a EventTarget {
        self.transmute()
    }

    fn mut_eventtarget<'a>(&'a mut self) -> &'a mut EventTarget {
        self.transmute_mut()
    }
}

impl DerivedWrapper for AbstractEventTarget {
    #[fixed_stack_segment]
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, vp: *mut JSVal) -> i32 {
        let wrapper = self.reflector().get_jsobject();
        if wrapper.is_not_null() {
            unsafe { *vp = RUST_OBJECT_TO_JSVAL(wrapper) };
            return 1;
        }
        unreachable!()
    }
}

impl Reflectable for AbstractEventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.mut_eventtarget().mut_reflector()
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        unreachable!()
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        self.eventtarget().GetParentObject(cx)
    }
}

impl EventTarget {
    pub fn new() -> EventTarget {
        EventTarget {
            reflector_: Reflector::new(),
            capturing_handlers: HashMap::new(),
            bubbling_handlers: HashMap::new(),
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn AddEventListener(&mut self,
                            ty: &DOMString,
                            listener: Option<EventListener>,
                            capture: bool) {
        // TODO: Handle adding a listener during event dispatch: should not be invoked during
        //       current phase.
        // (https://developer.mozilla.org/en-US/docs/Web/API/EventTarget.addEventListener#Adding_a_listener_during_event_dispatch)

        for listener in listener.iter() {
            let handlers = if capture {
                &mut self.capturing_handlers
            } else {
                &mut self.bubbling_handlers
            };
            let entry = handlers.find_or_insert_with(null_str_as_word_null(ty), |_| ~[]);
            if entry.position_elem(listener).is_none() {
                entry.push((*listener).clone());
            }
        }
    }

    pub fn RemoveEventListener(&mut self,
                               ty: &DOMString,
                               listener: Option<EventListener>,
                               capture: bool) {
        for listener in listener.iter() {
            let handlers = if capture {
                &mut self.capturing_handlers
            } else {
                &mut self.bubbling_handlers
            };
            let mut entry = handlers.find_mut(&null_str_as_word_null(ty));
            for entry in entry.mut_iter() {
                let position = entry.position_elem(listener);
                for &position in position.iter() {
                    entry.remove(position);
                }
            }
        }
    }

    pub fn DispatchEvent(&self, _abstract_self: AbstractEventTarget, event: AbstractEvent) -> Fallible<bool> {
        //FIXME: get proper |this| object

        let type_ = event.event().type_.clone();
        let maybe_handlers = self.capturing_handlers.find(&type_);
        for handlers in maybe_handlers.iter() {
            for handler in handlers.iter() {
                handler.HandleEvent__(event, eReportExceptions);
            }
        }
        if event.event().bubbles {
            let maybe_handlers = self.bubbling_handlers.find(&type_);
            for handlers in maybe_handlers.iter() {
                for handler in handlers.iter() {
                    handler.HandleEvent__(event, eReportExceptions);
                }
            }
        }
        Ok(!event.event().DefaultPrevented())
    }
}

impl Reflectable for EventTarget {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        EventTargetBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        // TODO(tkuehn): This only handles top-level pages. Needs to handle subframes.
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
