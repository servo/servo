/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard};
use style::stylesheets::{CssRule as StyleCssRule, CssRuleType};

use super::cssfontfacerule::CSSFontFaceRule;
use super::cssimportrule::CSSImportRule;
use super::csskeyframerule::CSSKeyframeRule;
use super::csskeyframesrule::CSSKeyframesRule;
use super::csslayerblockrule::CSSLayerBlockRule;
use super::csslayerstatementrule::CSSLayerStatementRule;
use super::cssmediarule::CSSMediaRule;
use super::cssnamespacerule::CSSNamespaceRule;
use super::cssnesteddeclarations::CSSNestedDeclarations;
use super::cssstylerule::CSSStyleRule;
use super::cssstylesheet::CSSStyleSheet;
use super::csssupportsrule::CSSSupportsRule;
use crate::dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSRule {
    reflector_: Reflector,
    parent_stylesheet: Dom<CSSStyleSheet>,

    /// Whether the parentStyleSheet attribute should return null.
    /// We keep parent_stylesheet in that case because insertRule needs it
    /// for the stylesheet’s base URL and namespace prefixes.
    parent_stylesheet_removed: Cell<bool>,
}

impl CSSRule {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(parent_stylesheet: &CSSStyleSheet) -> CSSRule {
        CSSRule {
            reflector_: Reflector::new(),
            parent_stylesheet: Dom::from_ref(parent_stylesheet),
            parent_stylesheet_removed: Cell::new(false),
        }
    }

    pub(crate) fn as_specific(&self) -> &dyn SpecificCSSRule {
        if let Some(rule) = self.downcast::<CSSStyleRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSFontFaceRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSKeyframesRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSMediaRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSNamespaceRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSKeyframeRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSImportRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSSupportsRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSLayerBlockRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSLayerStatementRule>() {
            rule as &dyn SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSNestedDeclarations>() {
            rule as &dyn SpecificCSSRule
        } else {
            unreachable!()
        }
    }

    // Given a StyleCssRule, create a new instance of a derived class of
    // CSSRule based on which rule it is
    pub(crate) fn new_specific(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        rule: StyleCssRule,
        can_gc: CanGc,
    ) -> DomRoot<CSSRule> {
        // be sure to update the match in as_specific when this is updated
        match rule {
            StyleCssRule::Import(s) => {
                DomRoot::upcast(CSSImportRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::Style(s) => {
                DomRoot::upcast(CSSStyleRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::FontFace(s) => {
                DomRoot::upcast(CSSFontFaceRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::FontFeatureValues(_) => unimplemented!(),
            StyleCssRule::CounterStyle(_) => unimplemented!(),
            StyleCssRule::Keyframes(s) => {
                DomRoot::upcast(CSSKeyframesRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::Media(s) => {
                DomRoot::upcast(CSSMediaRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::Namespace(s) => {
                DomRoot::upcast(CSSNamespaceRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::Supports(s) => {
                DomRoot::upcast(CSSSupportsRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::Page(_) => unreachable!(),
            StyleCssRule::Container(_) => unimplemented!(), // TODO
            StyleCssRule::Document(_) => unimplemented!(),  // TODO
            StyleCssRule::LayerBlock(s) => {
                DomRoot::upcast(CSSLayerBlockRule::new(window, parent_stylesheet, s, can_gc))
            },
            StyleCssRule::LayerStatement(s) => DomRoot::upcast(CSSLayerStatementRule::new(
                window,
                parent_stylesheet,
                s,
                can_gc,
            )),
            StyleCssRule::FontPaletteValues(_) => unimplemented!(), // TODO
            StyleCssRule::Property(_) => unimplemented!(),          // TODO
            StyleCssRule::Margin(_) => unimplemented!(),            // TODO
            StyleCssRule::Scope(_) => unimplemented!(),             // TODO
            StyleCssRule::StartingStyle(_) => unimplemented!(),     // TODO
            StyleCssRule::PositionTry(_) => unimplemented!(),       // TODO
            StyleCssRule::NestedDeclarations(s) => DomRoot::upcast(CSSNestedDeclarations::new(
                window,
                parent_stylesheet,
                s,
                can_gc,
            )),
        }
    }

    /// Sets owner sheet/rule to null
    pub(crate) fn detach(&self) {
        self.deparent();
        // should set parent rule to None when we add parent rule support
    }

    /// Sets owner sheet to null (and does the same for all children)
    pub(crate) fn deparent(&self) {
        self.parent_stylesheet_removed.set(true);
        // https://github.com/w3c/csswg-drafts/issues/722
        // Spec doesn't ask us to do this, but it makes sense
        // and browsers implement this behavior
        self.as_specific().deparent_children();
    }

    pub(crate) fn parent_stylesheet(&self) -> &CSSStyleSheet {
        &self.parent_stylesheet
    }

    pub(crate) fn shared_lock(&self) -> &SharedRwLock {
        self.parent_stylesheet.shared_lock()
    }

    pub(crate) fn update_rule(&self, style_rule: &StyleCssRule, guard: &SharedRwLockReadGuard) {
        match style_rule {
            StyleCssRule::Import(s) => {
                if let Some(rule) = self.downcast::<CSSImportRule>() {
                    rule.update_rule(s.clone());
                }
            },
            StyleCssRule::Style(s) => {
                if let Some(rule) = self.downcast::<CSSStyleRule>() {
                    rule.update_rule(s.clone(), guard);
                }
            },
            StyleCssRule::FontFace(s) => {
                if let Some(rule) = self.downcast::<CSSFontFaceRule>() {
                    rule.update_rule(s.clone());
                }
            },
            StyleCssRule::FontFeatureValues(_) => unimplemented!(),
            StyleCssRule::CounterStyle(_) => unimplemented!(),
            StyleCssRule::Keyframes(s) => {
                if let Some(rule) = self.downcast::<CSSKeyframesRule>() {
                    rule.update_rule(s.clone(), guard);
                }
            },
            StyleCssRule::Media(s) => {
                if let Some(rule) = self.downcast::<CSSMediaRule>() {
                    rule.update_rule(s.clone(), guard);
                }
            },
            StyleCssRule::Namespace(s) => {
                if let Some(rule) = self.downcast::<CSSNamespaceRule>() {
                    rule.update_rule(s.clone());
                }
            },
            StyleCssRule::Supports(s) => {
                if let Some(rule) = self.downcast::<CSSSupportsRule>() {
                    rule.update_rule(s.clone(), guard);
                }
            },
            StyleCssRule::Page(_) => unreachable!(),
            StyleCssRule::Container(_) => unimplemented!(), // TODO
            StyleCssRule::Document(_) => unimplemented!(),  // TODO
            StyleCssRule::LayerBlock(s) => {
                if let Some(rule) = self.downcast::<CSSLayerBlockRule>() {
                    rule.update_rule(s.clone(), guard);
                }
            },
            StyleCssRule::LayerStatement(s) => {
                if let Some(rule) = self.downcast::<CSSLayerStatementRule>() {
                    rule.update_rule(s.clone());
                }
            },
            StyleCssRule::FontPaletteValues(_) => unimplemented!(), // TODO
            StyleCssRule::Property(_) => unimplemented!(),          // TODO
            StyleCssRule::Margin(_) => unimplemented!(),            // TODO
            StyleCssRule::Scope(_) => unimplemented!(),             // TODO
            StyleCssRule::StartingStyle(_) => unimplemented!(),     // TODO
            StyleCssRule::PositionTry(_) => unimplemented!(),       // TODO
            StyleCssRule::NestedDeclarations(s) => {
                if let Some(rule) = self.downcast::<CSSNestedDeclarations>() {
                    rule.update_rule(s.clone(), guard);
                }
            },
        }
    }
}

impl CSSRuleMethods<crate::DomTypeHolder> for CSSRule {
    // https://drafts.csswg.org/cssom/#dom-cssrule-type
    fn Type(&self) -> u16 {
        let rule_type = self.as_specific().ty() as u16;
        // Per https://drafts.csswg.org/cssom/#dom-cssrule-type for constants > 15
        // we return 0.
        if rule_type > 15 { 0 } else { rule_type }
    }

    // https://drafts.csswg.org/cssom/#dom-cssrule-parentstylesheet
    fn GetParentStyleSheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        if self.parent_stylesheet_removed.get() {
            None
        } else {
            Some(DomRoot::from_ref(&*self.parent_stylesheet))
        }
    }

    // https://drafts.csswg.org/cssom/#dom-cssrule-csstext
    fn CssText(&self) -> DOMString {
        self.as_specific().get_css()
    }

    // https://drafts.csswg.org/cssom/#dom-cssrule-csstext
    fn SetCssText(&self, _: DOMString) {
        // do nothing
    }
}

pub(crate) trait SpecificCSSRule {
    fn ty(&self) -> CssRuleType;
    fn get_css(&self) -> DOMString;
    /// Remove parentStylesheet from all transitive children
    fn deparent_children(&self) {
        // most CSSRules do nothing here
    }
}
