/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::ProgressEventBinding;
use dom::bindings::codegen::InheritTypes::ProgressEventDerived;
use dom::bindings::error::Fallible;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, ProgressEventTypeId};
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct ProgressEvent {
    event: Event,
    length_computable: bool,
    loaded: u64,
    total: u64
}

impl ProgressEventDerived for Event {
    fn is_progressevent(&self) -> bool {
        self.type_id == ProgressEventTypeId
    }
}

impl ProgressEvent {
    pub fn new_inherited(length_computable: bool, loaded: u64, total: u64) -> ProgressEvent {
        ProgressEvent {
            event: Event::new_inherited(ProgressEventTypeId),
            length_computable: length_computable,
            loaded: loaded,
            total: total
        }
    }
    pub fn new(window: &JSRef<Window>, length_computable: bool,
                loaded: u64, total: u64) -> Temporary<ProgressEvent> {
        reflect_dom_object(box ProgressEvent::new_inherited(length_computable, loaded, total),
                           window,
                           ProgressEventBinding::Wrap)
    }
    pub fn Constructor(owner: &JSRef<Window>,
                       _type: DOMString,
                       init: &ProgressEventBinding::ProgressEventInit)
                       -> Fallible<Temporary<ProgressEvent>> {
        let ev = ProgressEvent::new(owner, init.lengthComputable, init.loaded, init.total);
        Ok(ev)
    }
}

pub trait ProgressEventMethods {
    fn LengthComputable(&self) -> bool;
    fn Loaded(&self) -> u64;
    fn Total(&self) -> u64;
}

impl<'a> ProgressEventMethods for JSRef<'a, ProgressEvent> {
    fn LengthComputable(&self) -> bool {
        self.length_computable
    }
    fn Loaded(&self) -> u64{
        self.loaded
    }
    fn Total(&self) -> u64 {
        self.total
    }
}

impl Reflectable for ProgressEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.event.mut_reflector()
    }
}