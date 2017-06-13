/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use dom::bindings::codegen::Bindings::MediaListBinding;
use dom::bindings::codegen::Bindings::MediaListBinding::MediaListMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use style::media_queries::{MediaQuery, parse_media_query_list};
use style::media_queries::MediaList as StyleMediaList;
use style::parser::ParserContext;
use style::shared_lock::{SharedRwLock, Locked};
use style::stylearc::Arc;
use style::stylesheets::CssRuleType;
use style_traits::{PARSING_MODE_DEFAULT, ToCss};

#[dom_struct]
pub struct MediaList {
    reflector_: Reflector,
    parent_stylesheet: JS<CSSStyleSheet>,
    #[ignore_heap_size_of = "Arc"]
    media_queries: Arc<Locked<StyleMediaList>>,
}

impl MediaList {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(parent_stylesheet: &CSSStyleSheet,
                         media_queries: Arc<Locked<StyleMediaList>>) -> MediaList {
        MediaList {
            parent_stylesheet: JS::from_ref(parent_stylesheet),
            reflector_: Reflector::new(),
            media_queries: media_queries,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               media_queries: Arc<Locked<StyleMediaList>>)
        -> Root<MediaList> {
        reflect_dom_object(box MediaList::new_inherited(parent_stylesheet, media_queries),
                           window,
                           MediaListBinding::Wrap)
    }

    fn shared_lock(&self) -> &SharedRwLock {
        &self.parent_stylesheet.style_stylesheet().shared_lock
    }
}

impl MediaListMethods for MediaList {
    // https://drafts.csswg.org/cssom/#dom-medialist-mediatext
    fn MediaText(&self) -> DOMString {
        let guard = self.shared_lock().read();
        DOMString::from(self.media_queries.read_with(&guard).to_css_string())
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-mediatext
    fn SetMediaText(&self, value: DOMString) {
        let mut guard = self.shared_lock().write();
        let mut media_queries = self.media_queries.write_with(&mut guard);
        // Step 2
        if value.is_empty() {
            // Step 1
            *media_queries = StyleMediaList::empty();
            return;
        }
        // Step 3
        let mut input = ParserInput::new(&value);
        let mut parser = Parser::new(&mut input);
        let global = self.global();
        let win = global.as_window();
        let url = win.get_url();
        let quirks_mode = win.Document().quirks_mode();
        let context = ParserContext::new_for_cssom(&url, win.css_error_reporter(), Some(CssRuleType::Media),
                                                   PARSING_MODE_DEFAULT,
                                                   quirks_mode);
        *media_queries = parse_media_query_list(&context, &mut parser);
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-length
    fn Length(&self) -> u32 {
        let guard = self.shared_lock().read();
        self.media_queries.read_with(&guard).media_queries.len() as u32
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-item
    fn Item(&self, index: u32) -> Option<DOMString> {
        let guard = self.shared_lock().read();
        self.media_queries.read_with(&guard).media_queries
        .get(index as usize).and_then(|query| {
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
        let mut input = ParserInput::new(&medium);
        let mut parser = Parser::new(&mut input);
        let global = self.global();
        let win = global.as_window();
        let url = win.get_url();
        let quirks_mode = win.Document().quirks_mode();
        let context = ParserContext::new_for_cssom(&url, win.css_error_reporter(), Some(CssRuleType::Media),
                                                   PARSING_MODE_DEFAULT,
                                                   quirks_mode);
        let m = MediaQuery::parse(&context, &mut parser);
        // Step 2
        if let Err(_) = m {
            return;
        }
        // Step 3
        let m_serialized = m.clone().unwrap().to_css_string();
        let mut guard = self.shared_lock().write();
        let mq = self.media_queries.write_with(&mut guard);
        let any = mq.media_queries.iter().any(|q| m_serialized == q.to_css_string());
        if any {
            return;
        }
        // Step 4
        mq.media_queries.push(m.unwrap());
    }

    // https://drafts.csswg.org/cssom/#dom-medialist-deletemedium
    fn DeleteMedium(&self, medium: DOMString) {
        // Step 1
        let mut input = ParserInput::new(&medium);
        let mut parser = Parser::new(&mut input);
        let global = self.global();
        let win = global.as_window();
        let url = win.get_url();
        let quirks_mode = win.Document().quirks_mode();
        let context = ParserContext::new_for_cssom(&url, win.css_error_reporter(), Some(CssRuleType::Media),
                                                   PARSING_MODE_DEFAULT,
                                                   quirks_mode);
        let m = MediaQuery::parse(&context, &mut parser);
        // Step 2
        if let Err(_) = m {
            return;
        }
        // Step 3
        let m_serialized = m.unwrap().to_css_string();
        let mut guard = self.shared_lock().write();
        let mut media_list = self.media_queries.write_with(&mut guard);
        let new_vec = media_list.media_queries.drain(..)
                                .filter(|q| m_serialized != q.to_css_string())
                                .collect();
        media_list.media_queries = new_vec;
    }
}
