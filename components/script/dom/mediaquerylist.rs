/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::MediaQueryListBinding::{self, MediaQueryListMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::weakref::{WeakRef, WeakRefVec};
use dom::document::Document;
use dom::eventtarget::EventTarget;
use euclid::scale_factor::ScaleFactor;
use js::jsapi::JSTracer;
use std::cell::Cell;
use std::rc::Rc;
use style;
use style::media_queries::{Device, MediaType};
use style_traits::{PagePx, ToCss, ViewportPx};

pub enum MediaQueryListMatchState {
    Same(bool),
    Changed(bool),
}

#[dom_struct]
pub struct MediaQueryList {
    eventtarget: EventTarget,
    document: JS<Document>,
    media_query_list: style::media_queries::MediaQueryList,
    last_match_state: Cell<Option<bool>>
}

impl MediaQueryList {
    fn new_inherited(document: &Document,
                     media_query_list: style::media_queries::MediaQueryList) -> MediaQueryList {
        MediaQueryList {
            eventtarget: EventTarget::new_inherited(),
            document: JS::from_ref(document),
            media_query_list: media_query_list,
            last_match_state: Cell::new(None),
        }
    }

    pub fn new(document: &Document,
               media_query_list: style::media_queries::MediaQueryList) -> Root<MediaQueryList> {
        reflect_dom_object(box MediaQueryList::new_inherited(document, media_query_list),
                           document.window(),
                           MediaQueryListBinding::Wrap)
    }
}

impl MediaQueryList {
    fn evaluate_changes(&self) -> MediaQueryListMatchState {
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
        if let Some(window_size) = self.document.window().window_size() {
            let viewport_size = window_size.visible_viewport;
            // TODO: support real ViewportPx, including zoom level
            // This information seems not to be tracked currently, so we assume
            // ViewportPx == PagePx
            let page_to_viewport: ScaleFactor<f32, PagePx, ViewportPx> = ScaleFactor::new(1.0);
            let device = Device::new(MediaType::Screen, viewport_size * page_to_viewport);
            self.media_query_list.evaluate(&device)
        } else {
            false
        }
    }
}

impl MediaQueryListMethods for MediaQueryList {
    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-media
    fn Media(&self) -> DOMString {
        let mut s = String::new();
        self.media_query_list.to_css(&mut s).unwrap();
        DOMString::from_string(s)
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
        self.upcast::<EventTarget>().AddEventListener(DOMString::from_string("change".to_owned()),
                                                      listener, false);
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-removelistener
    fn RemoveListener(&self, listener: Option<Rc<EventListener>>) {
        self.upcast::<EventTarget>().RemoveEventListener(DOMString::from_string("change".to_owned()),
                                                         listener, false);
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylist-onchange
    event_handler!(change, GetOnchange, SetOnchange);
}

#[derive(HeapSizeOf)]
pub struct WeakMediaQueryListVec {
    cell: DOMRefCell<WeakRefVec<MediaQueryList>>,
}

impl WeakMediaQueryListVec {
    /// Create a new vector of weak references to MediaQueryList
    pub fn new() -> Self {
        WeakMediaQueryListVec { cell: DOMRefCell::new(WeakRefVec::new()) }
    }

    pub fn push(&self, mql: &MediaQueryList) {
        self.cell.borrow_mut().push(WeakRef::new(mql));
    }

    /// Evaluate media query lists and report changes
    /// https://drafts.csswg.org/cssom-view/#evaluate-media-queries-and-report-changes
    pub fn evaluate_and_report_changes(&self) {
        for mql in self.cell.borrow().iter() {
            if let MediaQueryListMatchState::Changed(_) = mql.root().unwrap().evaluate_changes() {
                mql.root().unwrap().upcast::<EventTarget>().fire_event(atom!("change"));
            }
        }
    }
}

impl JSTraceable for WeakMediaQueryListVec {
    fn trace(&self, _: *mut JSTracer) {
        self.cell.borrow_mut().retain_alive()
    }
}
