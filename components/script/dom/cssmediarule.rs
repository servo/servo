/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, MediaRule};
use style_traits::ToCss;

use crate::dom::bindings::codegen::Bindings::CSSMediaRuleBinding::CSSMediaRuleMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssconditionrule::CSSConditionRule;
use crate::dom::cssrule::SpecificCSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::medialist::MediaList;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSMediaRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    mediarule: Arc<MediaRule>,
    medialist: MutNullableDom<MediaList>,
}

impl CSSMediaRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, mediarule: Arc<MediaRule>) -> CSSMediaRule {
        let list = mediarule.rules.clone();
        CSSMediaRule {
            cssconditionrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            mediarule,
            medialist: MutNullableDom::new(None),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        mediarule: Arc<MediaRule>,
    ) -> DomRoot<CSSMediaRule> {
        reflect_dom_object(
            Box::new(CSSMediaRule::new_inherited(parent_stylesheet, mediarule)),
            window,
            CanGc::note(),
        )
    }

    fn medialist(&self) -> DomRoot<MediaList> {
        self.medialist.or_init(|| {
            MediaList::new(
                self.global().as_window(),
                self.cssconditionrule.parent_stylesheet(),
                self.mediarule.media_queries.clone(),
            )
        })
    }

    /// <https://drafts.csswg.org/css-conditional-3/#the-cssmediarule-interface>
    pub(crate) fn get_condition_text(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        let list = self.mediarule.media_queries.read_with(&guard);
        list.to_css_string().into()
    }
}

impl SpecificCSSRule for CSSMediaRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Media
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        self.mediarule.to_css_string(&guard).into()
    }
}

impl CSSMediaRuleMethods<crate::DomTypeHolder> for CSSMediaRule {
    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-media
    fn Media(&self) -> DomRoot<MediaList> {
        self.medialist()
    }
}
