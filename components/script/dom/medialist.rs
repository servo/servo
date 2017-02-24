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
use dom_struct::dom_struct;
use parking_lot::RwLock;
use std::sync::Arc;
use style::media_queries::{MediaQuery, parse_media_query_list};
use style::media_queries::MediaList as StyleMediaList;
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
        DOMString::from(self.media_queries.read().to_css_string())
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-mediatext
    fn SetMediaText(&self, value: DOMString) {
        let mut media_queries = self.media_queries.write();
        // Step 2
        if value.is_empty() {
            // Step 1
            *media_queries = StyleMediaList::default();
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

    // https://drafts.csswg.org/cssom/#dom-medialist-appendmedium
    fn AppendMedium(&self, medium: DOMString) {
        // Step 1
        let mut parser = Parser::new(&medium);
        let m = MediaQuery::parse(&mut parser);
        // Step 2
        if let Err(_) = m {
            return;
        }
        // Step 3
        let m_serialized = m.clone().unwrap().to_css_string();
        let any = self.media_queries.read().media_queries.iter()
                                    .any(|q| m_serialized == q.to_css_string());
        if any {
            return;
        }
        // Step 4
        self.media_queries.write().media_queries.push(m.unwrap());
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-deletemedium
    fn DeleteMedium(&self, medium: DOMString) {
        // Step 1
        let mut parser = Parser::new(&medium);
        let m = MediaQuery::parse(&mut parser);
        // Step 2
        if let Err(_) = m {
            return;
        }
        // Step 3
        let m_serialized = m.unwrap().to_css_string();
        let mut media_list = self.media_queries.write();
        let new_vec = media_list.media_queries.drain(..)
                                .filter(|q| m_serialized != q.to_css_string())
                                .collect();
        media_list.media_queries = new_vec;
    }
}
