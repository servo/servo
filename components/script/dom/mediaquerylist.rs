/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::AddEventListenerOptions;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventListenerOptions;
use dom::bindings::codegen::Bindings::MediaQueryListBinding::{self, MediaQueryListMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::eventtarget::EventTarget;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::rc::Rc;
use style::media_queries::MediaList;
use style_traits::ToCss;
use typeholder::TypeHolderTrait;

pub enum MediaQueryListMatchState {
    Same(bool),
    Changed(bool),
}

#[dom_struct]
pub struct MediaQueryList<TH: TypeHolderTrait> {
    eventtarget: EventTarget<TH>,
    document: Dom<Document<TH>>,
    media_query_list: MediaList,
    last_match_state: Cell<Option<bool>>
}

impl<TH: TypeHolderTrait> MediaQueryList<TH> {
    fn new_inherited(document: &Document<TH>, media_query_list: MediaList) -> MediaQueryList<TH> {
        MediaQueryList {
            eventtarget: EventTarget::new_inherited(),
            document: Dom::from_ref(document),
            media_query_list: media_query_list,
            last_match_state: Cell::new(None),
        }
    }

    pub fn new(document: &Document<TH>, media_query_list: MediaList) -> DomRoot<MediaQueryList<TH>> {
        reflect_dom_object(Box::new(MediaQueryList::new_inherited(document, media_query_list)),
                           document.window(),
                           MediaQueryListBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> MediaQueryList<TH> {
    pub fn evaluate_changes(&self) -> MediaQueryListMatchState {
        let matches = self.evaluate();

        let result = if let Some(old_matches) = self.last_match_state.get() {
            if old_matches == matches {
                MediaQueryListMatchState::Same(matches)
            } else {
                MediaQueryListMatchState::Changed(matches)
            }
        } else {
            MediaQueryListMatchState::Changed(matches)
        };

        self.last_match_state.set(Some(matches));
        result
    }

    pub fn evaluate(&self) -> bool {
        self.document.device().map_or(false, |device| {
            self.media_query_list.evaluate(&device, self.document.quirks_mode())
        })
    }
}

impl<TH: TypeHolderTrait> MediaQueryListMethods<TH> for MediaQueryList<TH> {
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
    fn AddListener(&self, listener: Option<Rc<EventListener<TH>>>) {
        self.upcast::<EventTarget<TH>>().add_event_listener(
            DOMString::from_string("change".to_owned()),
            listener,
            AddEventListenerOptions { parent: EventListenerOptions { capture: false } },
        );
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-removelistener
    fn RemoveListener(&self, listener: Option<Rc<EventListener<TH>>>) {
        self.upcast::<EventTarget<TH>>().remove_event_listener(
            DOMString::from_string("change".to_owned()),
            listener,
            EventListenerOptions { capture: false },
        );
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-onchange
    event_handler!(change, GetOnchange, SetOnchange);
}
