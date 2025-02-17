/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{
    TextTrackKind, TextTrackMethods, TextTrackMode,
};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::texttrackcuelist::TextTrackCueList;
use crate::dom::texttracklist::TextTrackList;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct TextTrack {
    eventtarget: EventTarget,
    kind: TextTrackKind,
    label: String,
    language: String,
    id: String,
    mode: Cell<TextTrackMode>,
    cue_list: MutNullableDom<TextTrackCueList>,
    track_list: DomRefCell<Option<Dom<TextTrackList>>>,
}

impl TextTrack {
    pub(crate) fn new_inherited(
        id: DOMString,
        kind: TextTrackKind,
        label: DOMString,
        language: DOMString,
        mode: TextTrackMode,
        track_list: Option<&TextTrackList>,
    ) -> TextTrack {
        TextTrack {
            eventtarget: EventTarget::new_inherited(),
            kind,
            label: label.into(),
            language: language.into(),
            id: id.into(),
            mode: Cell::new(mode),
            cue_list: Default::default(),
            track_list: DomRefCell::new(track_list.map(Dom::from_ref)),
        }
    }

    pub(crate) fn new(
        window: &Window,
        id: DOMString,
        kind: TextTrackKind,
        label: DOMString,
        language: DOMString,
        mode: TextTrackMode,
        track_list: Option<&TextTrackList>,
    ) -> DomRoot<TextTrack> {
        reflect_dom_object(
            Box::new(TextTrack::new_inherited(
                id, kind, label, language, mode, track_list,
            )),
            window,
            CanGc::note(),
        )
    }

    pub(crate) fn get_cues(&self) -> DomRoot<TextTrackCueList> {
        self.cue_list
            .or_init(|| TextTrackCueList::new(self.global().as_window(), &[]))
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn add_track_list(&self, track_list: &TextTrackList) {
        *self.track_list.borrow_mut() = Some(Dom::from_ref(track_list));
    }

    pub(crate) fn remove_track_list(&self) {
        *self.track_list.borrow_mut() = None;
    }
}

impl TextTrackMethods<crate::DomTypeHolder> for TextTrack {
    // https://html.spec.whatwg.org/multipage/#dom-texttrack-kind
    fn Kind(&self) -> TextTrackKind {
        self.kind
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-label
    fn Label(&self) -> DOMString {
        DOMString::from(self.label.clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-language
    fn Language(&self) -> DOMString {
        DOMString::from(self.language.clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-id
    fn Id(&self) -> DOMString {
        DOMString::from(self.id.clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-mode
    fn Mode(&self) -> TextTrackMode {
        self.mode.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-mode
    fn SetMode(&self, value: TextTrackMode) {
        self.mode.set(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-cues
    fn GetCues(&self) -> Option<DomRoot<TextTrackCueList>> {
        match self.Mode() {
            TextTrackMode::Disabled => None,
            _ => Some(self.get_cues()),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-activecues
    fn GetActiveCues(&self) -> Option<DomRoot<TextTrackCueList>> {
        // XXX implement active cues logic
        //      https://github.com/servo/servo/issues/22314
        Some(TextTrackCueList::new(self.global().as_window(), &[]))
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-addcue
    fn AddCue(&self, cue: &TextTrackCue) -> ErrorResult {
        // FIXME(#22314, dlrobertson) add Step 1 & 2
        // Step 3
        if let Some(old_track) = cue.get_track() {
            // gecko calls RemoveCue when the given cue
            // has an associated track, but doesn't return
            // the error from it, so we wont either.
            if old_track.RemoveCue(cue).is_err() {
                warn!("Failed to remove cues for the added cue's text track");
            }
        }
        // Step 4
        self.get_cues().add(cue);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-texttrack-removecue
    fn RemoveCue(&self, cue: &TextTrackCue) -> ErrorResult {
        // Step 1
        let cues = self.get_cues();
        let index = match cues.find(cue) {
            Some(i) => Ok(i),
            None => Err(Error::NotFound),
        }?;
        // Step 2
        cues.remove(index);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#handler-texttrack-oncuechange
    event_handler!(cuechange, GetOncuechange, SetOncuechange);
}
