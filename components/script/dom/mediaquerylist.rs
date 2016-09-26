/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ToCss;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventListenerBinding::EventListener;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::MediaQueryListBinding::{self, MediaQueryListMethods};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::eventtarget::EventTarget;
use dom::window::Window;
use euclid::size::TypedSize2D;
use std::rc::Rc;
use style;
use style::media_queries::{Device, MediaType};
use util::geometry::au_rect_to_f32_rect;

#[dom_struct]
pub struct MediaQueryList {
    eventtarget: EventTarget,
    document: JS<Document>,
    media_query_list: style::media_queries::MediaQueryList,
}

impl MediaQueryList {
    fn new_inherited(document: &Document,
                     media_query_list: style::media_queries::MediaQueryList) -> MediaQueryList {
        MediaQueryList {
            eventtarget: EventTarget::new_inherited(),
            document: JS::from_ref(document),
            media_query_list: media_query_list,
        }
    }

    pub fn new(window: &Window, document: &Document,
               media_query_list: style::media_queries::MediaQueryList) -> Root<MediaQueryList> {
        reflect_dom_object(box MediaQueryList::new_inherited(document, media_query_list),
                           GlobalRef::Window(window),
                           MediaQueryListBinding::Wrap)
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
        let current_viewport = self.document.window().current_viewport();
        let viewport_size = TypedSize2D::from_untyped(&au_rect_to_f32_rect(current_viewport).size);
        let device = Device::new(MediaType::Screen, viewport_size);
        let result = self.media_query_list.evaluate(&device);
        result
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
