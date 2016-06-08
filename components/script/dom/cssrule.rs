/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::cssfontfacerule::CSSFontFaceRule;
use dom::cssimportrule::CSSImportRule;
use dom::csskeyframerule::CSSKeyframeRule;
use dom::csskeyframesrule::CSSKeyframesRule;
use dom::cssmediarule::CSSMediaRule;
use dom::cssnamespacerule::CSSNamespaceRule;
use dom::cssstylerule::CSSStyleRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::cssviewportrule::CSSViewportRule;
use dom::window::Window;
use std::cell::Cell;
use style::stylesheets::CssRule as StyleCssRule;


#[dom_struct]
pub struct CSSRule {
    reflector_: Reflector,
    parent_stylesheet: JS<CSSStyleSheet>,

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
            parent_stylesheet: JS::from_ref(parent_stylesheet),
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
        } else {
            unreachable!()
        }
    }

    // Given a StyleCssRule, create a new instance of a derived class of
    // CSSRule based on which rule it is
    pub fn new_specific(window: &Window, parent_stylesheet: &CSSStyleSheet,
                        rule: StyleCssRule) -> Root<CSSRule> {
        // be sure to update the match in as_specific when this is updated
        match rule {
            StyleCssRule::Import(s) => Root::upcast(CSSImportRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Style(s) => Root::upcast(CSSStyleRule::new(window, parent_stylesheet, s)),
            StyleCssRule::FontFace(s) => Root::upcast(CSSFontFaceRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Keyframes(s) => Root::upcast(CSSKeyframesRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Media(s) => Root::upcast(CSSMediaRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Namespace(s) => Root::upcast(CSSNamespaceRule::new(window, parent_stylesheet, s)),
            StyleCssRule::Viewport(s) => Root::upcast(CSSViewportRule::new(window, parent_stylesheet, s)),
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
}

impl CSSRuleMethods for CSSRule {
    // https://drafts.csswg.org/cssom/#dom-cssrule-type
    fn Type(&self) -> u16 {
        self.as_specific().ty()
    }

    // https://drafts.csswg.org/cssom/#dom-cssrule-parentstylesheet
    fn GetParentStyleSheet(&self) -> Option<Root<CSSStyleSheet>> {
        if self.parent_stylesheet_removed.get() {
            None
        } else {
            Some(Root::from_ref(&*self.parent_stylesheet))
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
