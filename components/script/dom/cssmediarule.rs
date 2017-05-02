/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use dom::bindings::codegen::Bindings::CSSMediaRuleBinding;
use dom::bindings::codegen::Bindings::CSSMediaRuleBinding::CSSMediaRuleMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssconditionrule::CSSConditionRule;
use dom::cssrule::SpecificCSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::medialist::MediaList;
use dom::window::Window;
use dom_struct::dom_struct;
use std::sync::Arc;
use style::media_queries::parse_media_query_list;
use style::parser::{LengthParsingMode, ParserContext};
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::{CssRuleType, MediaRule};
use style_traits::ToCss;

#[dom_struct]
pub struct CSSMediaRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_heap_size_of = "Arc"]
    mediarule: Arc<Locked<MediaRule>>,
    medialist: MutNullableJS<MediaList>,
}

impl CSSMediaRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, mediarule: Arc<Locked<MediaRule>>)
                     -> CSSMediaRule {
        let guard = parent_stylesheet.shared_lock().read();
        let list = mediarule.read_with(&guard).rules.clone();
        CSSMediaRule {
            cssconditionrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            mediarule: mediarule,
            medialist: MutNullableJS::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               mediarule: Arc<Locked<MediaRule>>) -> Root<CSSMediaRule> {
        reflect_dom_object(box CSSMediaRule::new_inherited(parent_stylesheet, mediarule),
                           window,
                           CSSMediaRuleBinding::Wrap)
    }

    fn medialist(&self) -> Root<MediaList> {
        self.medialist.or_init(|| {
            let guard = self.cssconditionrule.shared_lock().read();
            MediaList::new(self.global().as_window(),
                           self.cssconditionrule.parent_stylesheet(),
                           self.mediarule.read_with(&guard).media_queries.clone())
        })
    }

    /// https://drafts.csswg.org/css-conditional-3/#the-cssmediarule-interface
    pub fn get_condition_text(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        let rule = self.mediarule.read_with(&guard);
        let list = rule.media_queries.read_with(&guard);
        list.to_css_string().into()
    }

    /// https://drafts.csswg.org/css-conditional-3/#the-cssmediarule-interface
    pub fn set_condition_text(&self, text: DOMString) {
        let mut input = Parser::new(&text);
        let global = self.global();
        let win = global.as_window();
        let url = win.get_url();
        let quirks_mode = win.Document().quirks_mode();
        let context = ParserContext::new_for_cssom(&url, win.css_error_reporter(), Some(CssRuleType::Media),
                                                   LengthParsingMode::Default,
                                                   quirks_mode);
        let new_medialist = parse_media_query_list(&context, &mut input);
        let mut guard = self.cssconditionrule.shared_lock().write();

        // Clone an Arc because we can’t borrow `guard` twice at the same time.

        // FIXME(SimonSapin): allow access to multiple objects with one write guard?
        // Would need a set of usize pointer addresses or something,
        // the same object is not accessed more than once.
        let mqs = Arc::clone(&self.mediarule.write_with(&mut guard).media_queries);

        *mqs.write_with(&mut guard) = new_medialist;
    }
}

impl SpecificCSSRule for CSSMediaRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::MEDIA_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        self.mediarule.read_with(&guard).to_css_string(&guard).into()
    }
}

impl CSSMediaRuleMethods for CSSMediaRule {
    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-media
    fn Media(&self) -> Root<MediaList> {
        self.medialist()
    }
}
