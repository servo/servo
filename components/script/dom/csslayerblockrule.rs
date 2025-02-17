/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, LayerBlockRule};
use style_traits::ToCss;

use crate::dom::bindings::codegen::Bindings::CSSLayerBlockRuleBinding::CSSLayerBlockRuleMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssgroupingrule::CSSGroupingRule;
use crate::dom::cssrule::SpecificCSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSLayerBlockRule {
    cssgroupingrule: CSSGroupingRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    layerblockrule: Arc<LayerBlockRule>,
}

impl CSSLayerBlockRule {
    pub(crate) fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        layerblockrule: Arc<LayerBlockRule>,
    ) -> CSSLayerBlockRule {
        CSSLayerBlockRule {
            cssgroupingrule: CSSGroupingRule::new_inherited(
                parent_stylesheet,
                layerblockrule.rules.clone(),
            ),
            layerblockrule,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        layerblockrule: Arc<LayerBlockRule>,
        can_gc: CanGc,
    ) -> DomRoot<CSSLayerBlockRule> {
        reflect_dom_object(
            Box::new(CSSLayerBlockRule::new_inherited(
                parent_stylesheet,
                layerblockrule,
            )),
            window,
            can_gc,
        )
    }
}

impl SpecificCSSRule for CSSLayerBlockRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::LayerBlock
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssgroupingrule.shared_lock().read();
        self.layerblockrule.to_css_string(&guard).into()
    }
}

impl CSSLayerBlockRuleMethods<crate::DomTypeHolder> for CSSLayerBlockRule {
    /// <https://drafts.csswg.org/css-cascade-5/#dom-csslayerblockrule-name>
    fn Name(&self) -> DOMString {
        if let Some(name) = &self.layerblockrule.name {
            DOMString::from_string(name.to_css_string())
        } else {
            DOMString::new()
        }
    }
}
