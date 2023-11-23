/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::MediaRule;
use style_traits::ToCss;

use crate::dom::bindings::codegen::Bindings::CSSMediaRuleBinding::CSSMediaRuleMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssconditionrule::CSSConditionRule;
use crate::dom::cssrule::SpecificCSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::medialist::MediaList;
use crate::dom::window::Window;

#[dom_struct]
pub struct CSSMediaRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
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
