/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TextTrackCueListBinding::{
    self, TextTrackCueListMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct TextTrackCueList {
    reflector_: Reflector,
    dom_cues: DomRefCell<Vec<Dom<TextTrackCue>>>,
}

impl TextTrackCueList {
    pub fn new_inherited(cues: &[&TextTrackCue]) -> TextTrackCueList {
        TextTrackCueList {
            reflector_: Reflector::new(),
            dom_cues: DomRefCell::new(cues.iter().map(|g| Dom::from_ref(&**g)).collect()),
        }
    }

    pub fn new(window: &Window, cues: &[&TextTrackCue]) -> DomRoot<TextTrackCueList> {
        reflect_dom_object(
            Box::new(TextTrackCueList::new_inherited(cues)),
            window,
            TextTrackCueListBinding::Wrap,
        )
    }

    pub fn item(&self, idx: usize) -> Option<DomRoot<TextTrackCue>> {
        self.dom_cues
            .borrow()
            .get(idx)
            .map(|t| DomRoot::from_ref(&**t))
    }

    pub fn find(&self, cue: &TextTrackCue) -> Option<usize> {
        self.dom_cues
            .borrow()
            .iter()
            .enumerate()
            .filter(|(_, c)| **c == cue)
            .next()
            .map(|(i, _)| i)
    }

    pub fn add(&self, cue: &TextTrackCue) {
        // Only add a cue if it does not exist in the list
        if self.find(cue).is_none() {
            self.dom_cues.borrow_mut().push(Dom::from_ref(cue));
        }
    }

    pub fn remove(&self, idx: usize) {
        self.dom_cues.borrow_mut().remove(idx);
    }
}

impl TextTrackCueListMethods for TextTrackCueList {
    // https://html.spec.whatwg.org/multipage/#dom-texttrackcuelist-length
    fn Length(&self) -> u32 {
        self.dom_cues.borrow().len() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcuelist-item
    fn IndexedGetter(&self, idx: u32) -> Option<DomRoot<TextTrackCue>> {
        self.item(idx as usize)
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrackcuelist-getcuebyid
    fn GetCueById(&self, id: DOMString) -> Option<DomRoot<TextTrackCue>> {
        if id.is_empty() {
            None
        } else {
            self.dom_cues
                .borrow()
                .iter()
                .filter(|cue| cue.id() == id)
                .next()
                .map(|t| DomRoot::from_ref(&**t))
        }
    }
}
