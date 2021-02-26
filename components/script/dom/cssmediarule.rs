/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::CSSMediaRuleBinding::CSSMediaRuleMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssconditionrule::CSSConditionRule;
use crate::dom::cssrule::SpecificCSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::medialist::MediaList;
use crate::dom::window::Window;
use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::media_queries::MediaList as StyleMediaList;
use style::parser::ParserContext;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::{CssRuleType, MediaRule, Origin};
use style_traits::{ParsingMode, ToCss};

#[dom_struct]
pub struct CSSMediaRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_malloc_size_of = "Arc"]
    mediarule: Arc<Locked<MediaRule>>,
    medialist: MutNullableDom<MediaList>,
}

impl CSSMediaRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        mediarule: Arc<Locked<MediaRule>>,
    ) -> CSSMediaRule {
        let guard = parent_stylesheet.shared_lock().read();
        let list = mediarule.read_with(&guard).rules.clone();
        CSSMediaRule {
            cssconditionrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            mediarule: mediarule,
            medialist: MutNullableDom::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        mediarule: Arc<Locked<MediaRule>>,
    ) -> DomRoot<CSSMediaRule> {
        reflect_dom_object(
            Box::new(CSSMediaRule::new_inherited(parent_stylesheet, mediarule)),
            window,
        )
    }

    fn medialist(&self) -> DomRoot<MediaList> {
        self.medialist.or_init(|| {
            let guard = self.cssconditionrule.shared_lock().read();
            MediaList::new(
                self.global().as_window(),
                self.cssconditionrule.parent_stylesheet(),
                self.mediarule.read_with(&guard).media_queries.clone(),
            )
        })
    }

    /// <https://drafts.csswg.org/css-conditional-3/#the-cssmediarule-interface>
    pub fn get_condition_text(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        let rule = self.mediarule.read_with(&guard);
        let list = rule.media_queries.read_with(&guard);
        list.to_css_string().into()
    }

    /// <https://drafts.csswg.org/css-conditional-3/#the-cssmediarule-interface>
    pub fn set_condition_text(&self, text: DOMString) {
        let mut input = ParserInput::new(&text);
        let mut input = Parser::new(&mut input);
        let global = self.global();
        let window = global.as_window();
        let url = window.get_url();
        let quirks_mode = window.Document().quirks_mode();
        let context = ParserContext::new(
            Origin::Author,
            &url,
            Some(CssRuleType::Media),
            ParsingMode::DEFAULT,
            quirks_mode,
            window.css_error_reporter(),
            None,
        );

        let new_medialist = StyleMediaList::parse(&context, &mut input);
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
        use crate::dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::MEDIA_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        self.mediarule
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}

impl CSSMediaRuleMethods for CSSMediaRule {
    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-media
    fn Media(&self) -> DomRoot<MediaList> {
        self.medialist()
    }
}
