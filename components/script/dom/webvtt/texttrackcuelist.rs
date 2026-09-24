/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::TextTrackCueListBinding::TextTrackCueListMethods;
use crate::dom::bindings::root::{Dom, DomRoot, MutDom, UnrootedDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::texttrack::TextTrack;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct TextTrackCueList {
    reflector_: Reflector,
    text_track: MutDom<TextTrack>,
    dom_cues: DomRefCell<Vec<Dom<TextTrackCue>>>,
}

impl TextTrackCueList {
    pub(crate) fn new_inherited(text_track: &TextTrack) -> TextTrackCueList {
        TextTrackCueList {
            reflector_: Reflector::new(),
            text_track: MutDom::new(text_track),
            dom_cues: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        text_track: &TextTrack,
        window: &Window,
    ) -> DomRoot<TextTrackCueList> {
        reflect_dom_object_with_cx(
            Box::new(TextTrackCueList::new_inherited(text_track)),
            window,
            cx,
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

    pub(crate) fn add(&self, cx: &mut JSContext, cue: &TextTrackCue) {
        // Only add a cue if it does not exist in the list
        if self.find(cue).is_none() {
            self.dom_cues.borrow_mut().push(Dom::from_ref(cue));
            if let Some(track_list) = self.text_track.get().track_list() {
                track_list.notify_media_element_for_added_cue(cx, cue);
            }
        }
    }

    pub(crate) fn refresh_active_cues<'no_gc>(
        &self,
        no_gc: &'no_gc NoGC,
        other: UnrootedDom<'no_gc, TextTrackCueList>,
    ) {
        *self.dom_cues.safe_borrow_mut(no_gc) = other
            .cues(no_gc)
            .iter()
            .filter(|cue| cue.is_active())
            .map(|cue| cue.deref().clone())
            .collect();
    }

    pub(crate) fn remove(&self, idx: usize) {
        self.dom_cues.borrow_mut().remove(idx);
    }

    pub(crate) fn empty(&self) {
        self.dom_cues.borrow_mut().clear();
    }

    pub(crate) fn cues<'no_gc>(
        &self,
        no_gc: &'no_gc NoGC,
    ) -> Vec<UnrootedDom<'no_gc, TextTrackCue>> {
        self.dom_cues
            .borrow()
            .clone()
            .into_iter()
            .map(|track| track.as_unrooted(no_gc))
            .collect()
    }
}

impl TextTrackCueListMethods<crate::DomTypeHolder> for TextTrackCueList {
    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcuelist-length>
    fn Length(&self) -> u32 {
        self.dom_cues.borrow().len() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcuelist-item>
    fn IndexedGetter(&self, idx: u32) -> Option<DomRoot<TextTrackCue>> {
        self.item(idx as usize)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrackcuelist-getcuebyid>
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
