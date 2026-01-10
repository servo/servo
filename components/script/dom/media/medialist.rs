/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::media_queries::{MediaList as StyleMediaList, MediaQuery};
use style::parser::ParserContext;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::{CssRuleType, CustomMediaEvaluator, Origin, UrlExtraData};
use style_traits::{ParseError, ParsingMode, ToCss};

use crate::css::parser_context_for_document;
use crate::dom::bindings::codegen::Bindings::MediaListBinding::MediaListMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::css::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::node::NodeTraits;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaList {
    reflector_: Reflector,
    parent_stylesheet: Dom<CSSStyleSheet>,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    media_queries: RefCell<Arc<Locked<StyleMediaList>>>,
}

impl MediaList {
    pub(crate) fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        media_queries: Arc<Locked<StyleMediaList>>,
    ) -> MediaList {
        MediaList {
            parent_stylesheet: Dom::from_ref(parent_stylesheet),
            reflector_: Reflector::new(),
            media_queries: RefCell::new(media_queries),
        }
    }

    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        media_queries: Arc<Locked<StyleMediaList>>,
        can_gc: CanGc,
    ) -> DomRoot<MediaList> {
        reflect_dom_object(
            Box::new(MediaList::new_inherited(parent_stylesheet, media_queries)),
            window,
            can_gc,
        )
    }

    fn shared_lock(&self) -> &SharedRwLock {
        self.parent_stylesheet.shared_lock()
    }

    /// <https://drafts.csswg.org/cssom/#parse-a-media-query-list>
    pub(crate) fn parse_media_list(value: &str, window: &Window) -> StyleMediaList {
        if value.is_empty() {
            return StyleMediaList::empty();
        }
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let document = window.Document();
        let url_data = UrlExtraData(document.owner_global().api_base_url().get_arc());
        // FIXME(emilio): This looks somewhat fishy, since we use the context
        // only to parse the media query list, CssRuleType::Media doesn't make
        // much sense.
        let context = parser_context_for_document(
            &document,
            CssRuleType::Media,
            ParsingMode::DEFAULT,
            &url_data,
        );
        StyleMediaList::parse(&context, &mut parser)
    }

    /// <https://drafts.csswg.org/cssom/#parse-a-media-query>
    pub(crate) fn parse_media_query<'i>(
        value: &'i str,
        window: &Window,
    ) -> Result<MediaQuery, ParseError<'i>> {
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let document = window.Document();
        let url_data = UrlExtraData(document.owner_global().api_base_url().get_arc());
        let context = parser_context_for_document(
            &document,
            CssRuleType::Media,
            ParsingMode::DEFAULT,
            &url_data,
        );
        MediaQuery::parse(&context, &mut parser)
    }

    pub(crate) fn clone_media_list(&self) -> StyleMediaList {
        let guard = self.shared_lock().read();
        self.media_queries.borrow().read_with(&guard).clone()
    }

    pub(crate) fn update_media_list(&self, media_queries: Arc<Locked<StyleMediaList>>) {
        *self.media_queries.borrow_mut() = media_queries;
    }

    /// <https://html.spec.whatwg.org/multipage/#matches-the-environment>
    pub(crate) fn matches_environment(document: &Document, media_query: &str) -> bool {
        let quirks_mode = document.quirks_mode();
        let url_data = UrlExtraData(document.owner_global().api_base_url().get_arc());
        // FIXME(emilio): This should do the same that we do for other media
        // lists regarding the rule type and such, though it doesn't really
        // matter right now...
        //
        // Also, ParsingMode::all() is wrong, and should be DEFAULT.
        let context = ParserContext::new(
            Origin::Author,
            &url_data,
            Some(CssRuleType::Style),
            ParsingMode::all(),
            quirks_mode,
            /* namespaces = */ Default::default(),
            None,
            None,
        );
        let mut parser_input = ParserInput::new(media_query);
        let mut parser = Parser::new(&mut parser_input);
        let media_list = StyleMediaList::parse(&context, &mut parser);
        media_list.evaluate(
            document.window().layout().device(),
            quirks_mode,
            &mut CustomMediaEvaluator::none(),
        )
    }
}

impl MediaListMethods<crate::DomTypeHolder> for MediaList {
    /// <https://drafts.csswg.org/cssom/#dom-medialist-mediatext>
    fn MediaText(&self) -> DOMString {
        let guard = self.shared_lock().read();
        DOMString::from(
            self.media_queries
                .borrow()
                .read_with(&guard)
                .to_css_string(),
        )
    }

    /// <https://drafts.csswg.org/cssom/#dom-medialist-mediatext>
    fn SetMediaText(&self, value: DOMString) {
        self.parent_stylesheet.will_modify();
        let global = self.global();
        let mut guard = self.shared_lock().write();
        let media_queries_borrowed = self.media_queries.borrow();
        let media_queries = media_queries_borrowed.write_with(&mut guard);
        *media_queries = Self::parse_media_list(&value.str(), global.as_window());
        self.parent_stylesheet.notify_invalidations();
    }

    /// <https://drafts.csswg.org/cssom/#dom-medialist-length>
    fn Length(&self) -> u32 {
        let guard = self.shared_lock().read();
        self.media_queries
            .borrow()
            .read_with(&guard)
            .media_queries
            .len() as u32
    }

    /// <https://drafts.csswg.org/cssom/#dom-medialist-item>
    fn Item(&self, index: u32) -> Option<DOMString> {
        let guard = self.shared_lock().read();
        self.media_queries
            .borrow()
            .read_with(&guard)
            .media_queries
            .get(index as usize)
            .map(|query| query.to_css_string().into())
    }

    /// <https://drafts.csswg.org/cssom/#dom-medialist-item>
    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        self.Item(index)
    }

    /// <https://drafts.csswg.org/cssom/#dom-medialist-appendmedium>
    fn AppendMedium(&self, medium: DOMString) {
        // Step 1
        let global = self.global();
        let medium = medium.str();
        let m = Self::parse_media_query(&medium, global.as_window());
        // Step 2
        if m.is_err() {
            return;
        }
        // Step 3
        {
            let m_serialized = m.clone().unwrap().to_css_string();
            let guard = self.shared_lock().read();
            let any = self
                .media_queries
                .borrow()
                .read_with(&guard)
                .media_queries
                .iter()
                .any(|q| m_serialized == q.to_css_string());
            if any {
                return;
            }
        }
        // Step 4
        self.parent_stylesheet.will_modify();
        let mut guard = self.shared_lock().write();
        self.media_queries
            .borrow()
            .write_with(&mut guard)
            .media_queries
            .push(m.unwrap());
        self.parent_stylesheet.notify_invalidations();
    }

    /// <https://drafts.csswg.org/cssom/#dom-medialist-deletemedium>
    fn DeleteMedium(&self, medium: DOMString) {
        // Step 1
        let global = self.global();
        let medium = medium.str();
        let m = Self::parse_media_query(&medium, global.as_window());
        // Step 2
        if m.is_err() {
            return;
        }
        // Step 3
        self.parent_stylesheet.will_modify();
        let m_serialized = m.unwrap().to_css_string();
        let mut guard = self.shared_lock().write();
        let media_queries_borrowed = self.media_queries.borrow();
        let media_list = media_queries_borrowed.write_with(&mut guard);
        let new_vec = media_list
            .media_queries
            .drain(..)
            .filter(|q| m_serialized != q.to_css_string())
            .collect();
        media_list.media_queries = new_vec;
        self.parent_stylesheet.notify_invalidations();
    }
}
