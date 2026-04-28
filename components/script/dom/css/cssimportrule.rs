/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::context::JSContext;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::import_rule::ImportLayer;
use style::stylesheets::{CssRuleType, ImportRule};
use style_traits::ToCss;

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSImportRuleBinding::CSSImportRuleMethods;
use crate::dom::bindings::reflector::reflect_dom_object_with_cx;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct CSSImportRule {
    css_rule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    import_rule: RefCell<Arc<Locked<ImportRule>>>,
}

impl CSSImportRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        import_rule: Arc<Locked<ImportRule>>,
    ) -> Self {
        CSSImportRule {
            css_rule: CSSRule::new_inherited(parent_stylesheet),
            import_rule: RefCell::new(import_rule),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        import_rule: Arc<Locked<ImportRule>>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(Self::new_inherited(parent_stylesheet, import_rule)),
            window,
            cx,
        )
    }

    pub(crate) fn update_rule(&self, import_rule: Arc<Locked<ImportRule>>) {
        *self.import_rule.borrow_mut() = import_rule;
    }
}

impl SpecificCSSRule for CSSImportRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Import
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_rule.shared_lock().read();
        self.import_rule
            .borrow()
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}

impl CSSImportRuleMethods<crate::DomTypeHolder> for CSSImportRule {
    /// <https://drafts.csswg.org/cssom-1/#dom-cssimportrule-layername>
    fn GetLayerName(&self) -> Option<DOMString> {
        let guard = self.css_rule.shared_lock().read();
        match &self.import_rule.borrow().read_with(&guard).layer {
            ImportLayer::None => None,
            ImportLayer::Anonymous => Some(DOMString::new()),
            ImportLayer::Named(name) => Some(name.to_css_string().into()),
        }
    }
}
