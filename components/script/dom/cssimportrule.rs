/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::import_rule::ImportLayer;
use style::stylesheets::{CssRuleType, ImportRule};
use style_traits::ToCss;

use crate::dom::bindings::codegen::Bindings::CSSImportRuleBinding::CSSImportRuleMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrule::{CSSRule, SpecificCSSRule};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;

#[dom_struct]
pub struct CSSImportRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    import_rule: Arc<Locked<ImportRule>>,
}

impl CSSImportRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        import_rule: Arc<Locked<ImportRule>>,
    ) -> Self {
        CSSImportRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            import_rule,
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        import_rule: Arc<Locked<ImportRule>>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(parent_stylesheet, import_rule)),
            window,
        )
    }
}

impl SpecificCSSRule for CSSImportRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Import
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.import_rule
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}

impl CSSImportRuleMethods for CSSImportRule {
    /// <https://drafts.csswg.org/cssom-1/#dom-cssimportrule-layername>
    fn GetLayerName(&self) -> Option<DOMString> {
        let guard = self.cssrule.shared_lock().read();
        match &self.import_rule.read_with(&guard).layer {
            ImportLayer::None => None,
            ImportLayer::Anonymous => Some(DOMString::new()),
            ImportLayer::Named(name) => Some(DOMString::from_string(name.to_css_string())),
        }
    }
}
