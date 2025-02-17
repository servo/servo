/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use style::media_queries::MediaList;
use style_traits::ToCss;

use crate::dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use crate::dom::bindings::codegen::Bindings::EventTargetBinding::{
    AddEventListenerOptions, EventListenerOptions,
};
use crate::dom::bindings::codegen::Bindings::MediaQueryListBinding::MediaQueryListMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::eventtarget::EventTarget;
use crate::script_runtime::CanGc;

pub(crate) enum MediaQueryListMatchState {
    Same,
    Changed,
}

#[dom_struct]
pub(crate) struct MediaQueryList {
    eventtarget: EventTarget,
    document: Dom<Document>,
    #[no_trace]
    media_query_list: MediaList,
    last_match_state: Cell<Option<bool>>,
}

impl MediaQueryList {
    fn new_inherited(document: &Document, media_query_list: MediaList) -> MediaQueryList {
        MediaQueryList {
            eventtarget: EventTarget::new_inherited(),
            document: Dom::from_ref(document),
            media_query_list,
            last_match_state: Cell::new(None),
        }
    }

    pub(crate) fn new(
        document: &Document,
        media_query_list: MediaList,
        can_gc: CanGc,
    ) -> DomRoot<MediaQueryList> {
        reflect_dom_object(
            Box::new(MediaQueryList::new_inherited(document, media_query_list)),
            document.window(),
            can_gc,
        )
    }
}

impl MediaQueryList {
    pub(crate) fn evaluate_changes(&self) -> MediaQueryListMatchState {
        let matches = self.evaluate();

        let result = if let Some(old_matches) = self.last_match_state.get() {
            if old_matches == matches {
                MediaQueryListMatchState::Same
            } else {
                MediaQueryListMatchState::Changed
            }
        } else {
            MediaQueryListMatchState::Changed
        };

        self.last_match_state.set(Some(matches));
        result
    }

    pub(crate) fn evaluate(&self) -> bool {
        let quirks_mode = self.document.quirks_mode();
        self.media_query_list
            .evaluate(self.document.window().layout().device(), quirks_mode)
    }
}

impl MediaQueryListMethods<crate::DomTypeHolder> for MediaQueryList {
    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-media
    fn Media(&self) -> DOMString {
        self.media_query_list.to_css_string().into()
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-matches
    fn Matches(&self) -> bool {
        match self.last_match_state.get() {
            None => self.evaluate(),
            Some(state) => state,
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-addlistener
    fn AddListener(&self, listener: Option<Rc<EventListener>>) {
        self.upcast::<EventTarget>().add_event_listener(
            DOMString::from_string("change".to_owned()),
            listener,
            AddEventListenerOptions {
                parent: EventListenerOptions { capture: false },
                once: false,
            },
        );
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-removelistener
    fn RemoveListener(&self, listener: Option<Rc<EventListener>>) {
        self.upcast::<EventTarget>().remove_event_listener(
            DOMString::from_string("change".to_owned()),
            listener,
            EventListenerOptions { capture: false },
        );
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-onchange
    event_handler!(change, GetOnchange, SetOnchange);
}
