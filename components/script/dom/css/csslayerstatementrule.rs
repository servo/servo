/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, LayerStatementRule};
use style_traits::ToCss;

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSLayerStatementRuleBinding::CSSLayerStatementRuleMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[dom_struct]
pub(crate) struct CSSLayerStatementRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    layerstatementrule: RefCell<Arc<LayerStatementRule>>,
}

impl CSSLayerStatementRule {
    pub(crate) fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        layerstatementrule: Arc<LayerStatementRule>,
    ) -> CSSLayerStatementRule {
        CSSLayerStatementRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            layerstatementrule: RefCell::new(layerstatementrule),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        layerstatementrule: Arc<LayerStatementRule>,
        can_gc: CanGc,
    ) -> DomRoot<CSSLayerStatementRule> {
        reflect_dom_object(
            Box::new(CSSLayerStatementRule::new_inherited(
                parent_stylesheet,
                layerstatementrule,
            )),
            window,
            can_gc,
        )
    }

    pub(crate) fn update_rule(&self, layerstatementrule: Arc<LayerStatementRule>) {
        *self.layerstatementrule.borrow_mut() = layerstatementrule;
    }
}

impl SpecificCSSRule for CSSLayerStatementRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::LayerStatement
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.layerstatementrule
            .borrow()
            .to_css_string(&guard)
            .into()
    }
}

impl CSSLayerStatementRuleMethods<crate::DomTypeHolder> for CSSLayerStatementRule {
    /// <https://drafts.csswg.org/css-cascade-5/#dom-csslayerstatementrule-namelist>
    fn NameList(&self, cx: SafeJSContext, can_gc: CanGc, retval: MutableHandleValue) {
        let names: Vec<DOMString> = self
            .layerstatementrule
            .borrow()
            .names
            .iter()
            .map(|name| DOMString::from_string(name.to_css_string()))
            .collect();
        to_frozen_array(names.as_slice(), cx, retval, can_gc)
    }
}
