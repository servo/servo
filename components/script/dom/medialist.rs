/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::default::Default;
use cssparser::Parser;
use dom::bindings::codegen::Bindings::MediaListBinding;
use dom::bindings::codegen::Bindings::MediaListBinding::MediaListMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::media_queries::MediaList as StyleMediaList;
use style::media_queries::parse_media_query_list;
use style_traits::ToCss;

#[dom_struct]
pub struct MediaList {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Arc"]
    media_queries: Arc<RwLock<StyleMediaList>>,
}

impl MediaList {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(media_queries: Arc<RwLock<StyleMediaList>>) -> MediaList {
        MediaList {
            reflector_: Reflector::new(),
            media_queries: media_queries,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, media_queries: Arc<RwLock<StyleMediaList>>)
        -> Root<MediaList> {
        reflect_dom_object(box MediaList::new_inherited(media_queries),
                           window,
                           MediaListBinding::Wrap)
    }

}

impl MediaListMethods for MediaList {
    // https://drafts.csswg.org/cssom/#dom-medialist-mediatext
    fn MediaText(&self) -> DOMString {
        let mut s = String::new();
        self.media_queries.read().to_css(&mut s).unwrap();
        DOMString::from_string(s)
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-mediatext
    fn SetMediaText(&self, value: DOMString) {
        // Step 1
        let mut media_queries = self.media_queries.write();
        *media_queries = StyleMediaList::default();
        // Step 2
        if value.is_empty() {
            return;
        }
        // Step 3
        let mut parser = Parser::new(&value);
        *media_queries = parse_media_query_list(&mut parser);
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-length
    fn Length(&self) -> u32 {
        self.media_queries.read().media_queries.len() as u32
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-item
    fn Item(&self, index: u32) -> Option<DOMString> {
        self.media_queries.read().media_queries.get(index as usize)
                                            .and_then(|query| {
            let mut s = String::new();
            query.to_css(&mut s).unwrap();
            Some(DOMString::from_string(s))

        })
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-item
    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        self.Item(index)
    }
}
