/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use style::stylesheets::{CssRuleType, MediaRule};
use style_traits::ToCss;

use super::cssconditionrule::CSSConditionRule;
use super::cssrule::SpecificCSSRule;
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSMediaRuleBinding::CSSMediaRuleMethods;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::medialist::MediaList;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSMediaRule {
    css_condition_rule: CSSConditionRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    media_rule: RefCell<Arc<MediaRule>>,
    media_list: MutNullableDom<MediaList>,
}

impl CSSMediaRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, mediarule: Arc<MediaRule>) -> CSSMediaRule {
        let list = mediarule.rules.clone();
        CSSMediaRule {
            css_condition_rule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            media_rule: RefCell::new(mediarule),
            media_list: MutNullableDom::new(None),
        }
    }

    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        mediarule: Arc<MediaRule>,
        can_gc: CanGc,
    ) -> DomRoot<CSSMediaRule> {
        reflect_dom_object(
            Box::new(CSSMediaRule::new_inherited(parent_stylesheet, mediarule)),
            window,
            can_gc,
        )
    }

    fn medialist(&self, can_gc: CanGc) -> DomRoot<MediaList> {
        self.media_list.or_init(|| {
            MediaList::new(
                self.global().as_window(),
                self.css_condition_rule.parent_stylesheet(),
                self.media_rule.borrow().media_queries.clone(),
                can_gc,
            )
        })
    }

    /// <https://drafts.csswg.org/css-conditional-3/#the-cssmediarule-interface>
    pub(crate) fn get_condition_text(&self) -> DOMString {
        let guard = self.css_condition_rule.shared_lock().read();
        self.media_rule
            .borrow()
            .media_queries
            .read_with(&guard)
            .to_css_string()
            .into()
    }

    pub(crate) fn update_rule(&self, mediarule: Arc<MediaRule>, guard: &SharedRwLockReadGuard) {
        self.css_condition_rule
            .update_rules(mediarule.rules.clone(), guard);
        if let Some(medialist) = self.media_list.get() {
            medialist.update_media_list(mediarule.media_queries.clone());
        }
        *self.media_rule.borrow_mut() = mediarule;
    }
}

impl SpecificCSSRule for CSSMediaRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Media
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_condition_rule.shared_lock().read();
        self.media_rule.borrow().to_css_string(&guard).into()
    }
}

impl CSSMediaRuleMethods<crate::DomTypeHolder> for CSSMediaRule {
    /// <https://drafts.csswg.org/cssom/#dom-cssgroupingrule-media>
    fn Media(&self, can_gc: CanGc) -> DomRoot<MediaList> {
        self.medialist(can_gc)
    }
}
