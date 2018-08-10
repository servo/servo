/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use dom::bindings::codegen::Bindings::CSSMediaRuleBinding;
use dom::bindings::codegen::Bindings::CSSMediaRuleBinding::CSSMediaRuleMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::cssconditionrule::CSSConditionRule;
use dom::cssrule::SpecificCSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::medialist::MediaList;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::media_queries::MediaList as StyleMediaList;
use style::parser::ParserContext;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::{CssRuleType, MediaRule};
use style_traits::{ParsingMode, ToCss};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct CSSMediaRule<TH: TypeHolderTrait> {
    cssconditionrule: CSSConditionRule<TH>,
    #[ignore_malloc_size_of = "Arc"]
    mediarule: Arc<Locked<MediaRule>>,
    medialist: MutNullableDom<MediaList<TH>>,
}

impl<TH: TypeHolderTrait> CSSMediaRule<TH> {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet<TH>, mediarule: Arc<Locked<MediaRule>>)
                     -> Self {
        let guard = parent_stylesheet.shared_lock().read();
        let list = mediarule.read_with(&guard).rules.clone();
        CSSMediaRule {
            cssconditionrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            mediarule: mediarule,
            medialist: MutNullableDom::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, parent_stylesheet: &CSSStyleSheet<TH>,
               mediarule: Arc<Locked<MediaRule>>) -> DomRoot<Self> {
        reflect_dom_object(Box::new(CSSMediaRule::new_inherited(parent_stylesheet, mediarule)),
                           window,
                           CSSMediaRuleBinding::Wrap)
    }

    fn medialist(&self) -> DomRoot<MediaList<TH>> {
        self.medialist.or_init(|| {
            let guard = self.cssconditionrule.shared_lock().read();
            MediaList::new(self.global().as_window(),
                           self.cssconditionrule.parent_stylesheet(),
                           self.mediarule.read_with(&guard).media_queries.clone())
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
        let context = ParserContext::new_for_cssom(
            &url,
            Some(CssRuleType::Media),
            ParsingMode::DEFAULT,
            quirks_mode,
            window.css_error_reporter(),
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

impl<TH: TypeHolderTrait> SpecificCSSRule for CSSMediaRule<TH> {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::MEDIA_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        self.mediarule.read_with(&guard).to_css_string(&guard).into()
    }
}

impl<TH: TypeHolderTrait> CSSMediaRuleMethods<TH> for CSSMediaRule<TH> {
    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-media
    fn Media(&self) -> DomRoot<MediaList<TH>> {
        self.medialist()
    }
}
