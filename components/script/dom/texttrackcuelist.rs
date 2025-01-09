/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TextTrackCueListBinding::TextTrackCueListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct TextTrackCueList {
    reflector_: Reflector,
    dom_cues: DomRefCell<Vec<Dom<TextTrackCue>>>,
}

impl TextTrackCueList {
    pub(crate) fn new_inherited(cues: &[&TextTrackCue]) -> TextTrackCueList {
        TextTrackCueList {
            reflector_: Reflector::new(),
            dom_cues: DomRefCell::new(cues.iter().map(|g| Dom::from_ref(&**g)).collect()),
        }
    }

    pub(crate) fn new(window: &Window, cues: &[&TextTrackCue]) -> DomRoot<TextTrackCueList> {
        reflect_dom_object(
            Box::new(TextTrackCueList::new_inherited(cues)),
            window,
            CanGc::note(),
        )
    }

    pub(crate) fn item(&self, idx: usize) -> Option<DomRoot<TextTrackCue>> {
        self.dom_cues
            .borrow()
            .get(idx)
            .map(|t| DomRoot::from_ref(&**t))
    }

    pub(crate) fn find(&self, cue: &TextTrackCue) -> Option<usize> {
        self.dom_cues
            .borrow()
            .iter()
            .enumerate()
            .find(|(_, c)| **c == cue)
            .map(|(i, _)| i)
    }

    pub(crate) fn add(&self, cue: &TextTrackCue) {
        // Only add a cue if it does not exist in the list
        if self.find(cue).is_none() {
            self.dom_cues.borrow_mut().push(Dom::from_ref(cue));
        }
    }

    pub(crate) fn remove(&self, idx: usize) {
        self.dom_cues.borrow_mut().remove(idx);
    }
}

impl TextTrackCueListMethods<crate::DomTypeHolder> for TextTrackCueList {
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
                .find(|cue| cue.id() == id)
                .map(|t| DomRoot::from_ref(&**t))
        }
    }
}
