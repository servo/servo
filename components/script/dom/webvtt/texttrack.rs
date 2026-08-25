/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{
    TextTrackKind, TextTrackMethods, TextTrackMode,
};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmltrackelement::HTMLTrackElement;
use crate::dom::texttrackcue::TextTrackCue;
use crate::dom::texttrackcuelist::TextTrackCueList;
use crate::dom::texttracklist::TextTrackList;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct TextTrack {
    eventtarget: EventTarget,
    /// <https://html.spec.whatwg.org/multipage/#text-track-kind>
    kind: TextTrackKind,
    /// <https://html.spec.whatwg.org/multipage/#text-track-label>
    label: String,
    /// <https://html.spec.whatwg.org/multipage/#text-track-language>
    language: String,
    id: String,
    /// <https://html.spec.whatwg.org/multipage/#text-track-mode>
    mode: Cell<TextTrackMode>,
    /// <https://html.spec.whatwg.org/multipage/#text-track-list-of-cues>
    cue_list: MutNullableDom<TextTrackCueList>,
    track_list: DomRefCell<Option<Dom<TextTrackList>>>,
    associated_track: DomRefCell<Option<Dom<HTMLTrackElement>>>,
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
            associated_track: Default::default(),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        id: DOMString,
        kind: TextTrackKind,
        label: DOMString,
        language: DOMString,
        mode: TextTrackMode,
        track_list: Option<&TextTrackList>,
    ) -> DomRoot<TextTrack> {
        reflect_dom_object_with_cx(
            Box::new(TextTrack::new_inherited(
                id, kind, label, language, mode, track_list,
            )),
            window,
            cx,
        )
    }

    pub(crate) fn get_cues(&self, cx: &mut JSContext) -> DomRoot<TextTrackCueList> {
        self.cue_list
            .or_init(|| TextTrackCueList::new(cx, self.global().as_window(), &[]))
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

    pub(crate) fn associated_track(&self) -> Option<DomRoot<HTMLTrackElement>> {
        self.associated_track
            .borrow()
            .as_ref()
            .map(|track| DomRoot::from_ref(&**track))
    }

    pub(crate) fn set_associated_track(&self, track_element: &HTMLTrackElement) {
        *self.associated_track.borrow_mut() = Some(Dom::from_ref(track_element));
    }

    pub(crate) fn empty_cue_list(&self) {
        if let Some(cue_list) = self.cue_list.get() {
            cue_list.empty();
        }
    }

    pub(crate) fn set_text_track_mode(&self, cx: &mut JSContext, value: TextTrackMode) {
        if self.mode.get() == value {
            return;
        }
        self.mode.set(value);
        // https://html.spec.whatwg.org/multipage/#sourcing-out-of-band-text-tracks:start-the-track-processing-model
        // > The text track has its text track mode changed.
        if let Some(track_element) = self.associated_track.borrow().as_ref() {
            track_element.start_the_track_processing_model(cx);
        }
    }
}

impl TextTrackMethods<crate::DomTypeHolder> for TextTrack {
    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-kind>
    fn Kind(&self) -> TextTrackKind {
        self.kind
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-label>
    fn Label(&self) -> DOMString {
        DOMString::from(self.label.clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-language>
    fn Language(&self) -> DOMString {
        DOMString::from(self.language.clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-id>
    fn Id(&self) -> DOMString {
        DOMString::from(self.id.clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-mode>
    fn Mode(&self) -> TextTrackMode {
        self.mode.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-mode>
    fn SetMode(&self, cx: &mut JSContext, value: TextTrackMode) {
        self.set_text_track_mode(cx, value)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-cues>
    fn GetCues(&self, cx: &mut JSContext) -> Option<DomRoot<TextTrackCueList>> {
        match self.Mode() {
            TextTrackMode::Disabled => None,
            _ => Some(self.get_cues(cx)),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-activecues>
    fn GetActiveCues(&self, cx: &mut JSContext) -> Option<DomRoot<TextTrackCueList>> {
        // XXX implement active cues logic
        //      https://github.com/servo/servo/issues/22314
        Some(TextTrackCueList::new(cx, self.global().as_window(), &[]))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-addcue>
    fn AddCue(&self, cx: &mut JSContext, cue: &TextTrackCue) -> ErrorResult {
        // FIXME(#22314, dlrobertson) add Step 1 & 2
        // Step 3
        if let Some(old_track) = cue.get_track() {
            // gecko calls RemoveCue when the given cue
            // has an associated track, but doesn't return
            // the error from it, so we wont either.
            if old_track.RemoveCue(cx, cue).is_err() {
                warn!("Failed to remove cues for the added cue's text track");
            }
        }
        // Step 4
        self.get_cues(cx).add(cue);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-texttrack-removecue>
    fn RemoveCue(&self, cx: &mut JSContext, cue: &TextTrackCue) -> ErrorResult {
        // Step 1
        let cues = self.get_cues(cx);
        let index = match cues.find(cue) {
            Some(i) => Ok(i),
            None => Err(Error::NotFound(None)),
        }?;
        // Step 2
        cues.remove(index);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#handler-texttrack-oncuechange
    event_handler!(cuechange, GetOncuechange, SetOncuechange);
}
