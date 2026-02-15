/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, SharedRwLockReadGuard, ToCssWithGuard};
use style::stylesheets::{CssRuleType, CssRules, LayerBlockRule};
use style_traits::ToCss;

use super::cssgroupingrule::CSSGroupingRule;
use super::cssrule::SpecificCSSRule;
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSLayerBlockRuleBinding::CSSLayerBlockRuleMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSLayerBlockRule {
    css_grouping_rule: CSSGroupingRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    layer_block_rule: RefCell<Arc<LayerBlockRule>>,
}

impl CSSLayerBlockRule {
    pub(crate) fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        layerblockrule: Arc<LayerBlockRule>,
    ) -> CSSLayerBlockRule {
        CSSLayerBlockRule {
            css_grouping_rule: CSSGroupingRule::new_inherited(parent_stylesheet),
            layer_block_rule: RefCell::new(layerblockrule),
        }
    }

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

    pub(crate) fn clone_rules(&self) -> Arc<Locked<CssRules>> {
        self.layer_block_rule.borrow().rules.clone()
    }

    pub(crate) fn update_rule(
        &self,
        layerblockrule: Arc<LayerBlockRule>,
        guard: &SharedRwLockReadGuard,
    ) {
        self.css_grouping_rule
            .update_rules(&layerblockrule.rules, guard);
        *self.layer_block_rule.borrow_mut() = layerblockrule;
    }
}

impl SpecificCSSRule for CSSLayerBlockRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::LayerBlock
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_grouping_rule.shared_lock().read();
        self.layer_block_rule.borrow().to_css_string(&guard).into()
    }
}

impl CSSLayerBlockRuleMethods<crate::DomTypeHolder> for CSSLayerBlockRule {
    /// <https://drafts.csswg.org/css-cascade-5/#dom-csslayerblockrule-name>
    fn Name(&self) -> DOMString {
        if let Some(name) = &self.layer_block_rule.borrow().name {
            DOMString::from_string(name.to_css_string())
        } else {
            DOMString::new()
        }
    }
}
