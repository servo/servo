/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::Reflector;
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::cssfontfacerule::CSSFontFaceRule;
use dom::cssimportrule::CSSImportRule;
use dom::csskeyframerule::CSSKeyframeRule;
use dom::csskeyframesrule::CSSKeyframesRule;
use dom::cssmediarule::CSSMediaRule;
use dom::cssnamespacerule::CSSNamespaceRule;
use dom::cssstylerule::CSSStyleRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::csssupportsrule::CSSSupportsRule;
use dom::cssviewportrule::CSSViewportRule;
use dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;
use style::shared_lock::SharedRwLock;
use style::stylesheets::CssRule as StyleCssRule;


#[dom_struct]
pub struct CSSRule {
    reflector_: Reflector,
    parent_stylesheet: Dom<CSSStyleSheet>,

    /// Whether the parentStyleSheet attribute should return null.
    /// We keep parent_stylesheet in that case because insertRule needs it
    /// for the stylesheet’s base URL and namespace prefixes.
    parent_stylesheet_removed: Cell<bool>,
}

impl CSSRule {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(parent_stylesheet: &CSSStyleSheet) -> CSSRule {
        CSSRule {
            reflector_: Reflector::new(),
            parent_stylesheet: Dom::from_ref(parent_stylesheet),
            parent_stylesheet_removed: Cell::new(false),
        }
    }

    pub fn as_specific(&self) -> &SpecificCSSRule {
        if let Some(rule) = self.downcast::<CSSStyleRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSFontFaceRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSKeyframesRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSMediaRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSNamespaceRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSViewportRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSKeyframeRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSImportRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSSupportsRule>() {
            rule as &SpecificCSSRule
        } else {
            unreachable!()
        }
    }

    // Given a StyleCssRule, create a new instance of a derived class of
    // CSSRule based on which rule it is
    pub fn new_specific(window: &Window, parent_stylesheet: &CSSStyleSheet,
                        rule: StyleCssRule) -> DomRoot<CSSRule> {
        // be sure to update the match in as_specific when this is updated
        match rule {
            StyleCssRule::Import(s) => DomRoot::upcast(CSSImportRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Style(s) => DomRoot::upcast(CSSStyleRule::new(window, parent_stylesheet, s)),
            StyleCssRule::FontFace(s) => DomRoot::upcast(CSSFontFaceRule::new(window, parent_stylesheet, s)),
            StyleCssRule::FontFeatureValues(_) => unimplemented!(),
            StyleCssRule::CounterStyle(_) => unimplemented!(),
            StyleCssRule::Keyframes(s) => DomRoot::upcast(CSSKeyframesRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Media(s) => DomRoot::upcast(CSSMediaRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Namespace(s) => DomRoot::upcast(CSSNamespaceRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Viewport(s) => DomRoot::upcast(CSSViewportRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Supports(s) => DomRoot::upcast(CSSSupportsRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Page(_) => unreachable!(),
            StyleCssRule::Document(_) => unimplemented!(), // TODO
        }
    }

    /// Sets owner sheet/rule to null
    pub fn detach(&self) {
        self.deparent();
        // should set parent rule to None when we add parent rule support
    }

    /// Sets owner sheet to null (and does the same for all children)
    pub fn deparent(&self) {
        self.parent_stylesheet_removed.set(true);
        // https://github.com/w3c/csswg-drafts/issues/722
        // Spec doesn't ask us to do this, but it makes sense
        // and browsers implement this behavior
        self.as_specific().deparent_children();
    }

    pub fn parent_stylesheet(&self) -> &CSSStyleSheet {
        &self.parent_stylesheet
    }

    pub fn shared_lock(&self) -> &SharedRwLock {
        &self.parent_stylesheet.style_stylesheet().shared_lock
    }
}

impl CSSRuleMethods for CSSRule {
    // https://drafts.csswg.org/cssom/#dom-cssrule-type
    fn Type(&self) -> u16 {
        self.as_specific().ty()
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

pub trait SpecificCSSRule {
    fn ty(&self) -> u16;
    fn get_css(&self) -> DOMString;
    /// Remove parentStylesheet from all transitive children
    fn deparent_children(&self) {
        // most CSSRules do nothing here
    }
}
