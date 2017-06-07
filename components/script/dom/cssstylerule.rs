/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser as CssParser, ParserInput as CssParserInput};
use cssparser::ToCss;
use dom::bindings::codegen::Bindings::CSSStyleRuleBinding::{self, CSSStyleRuleMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use selectors::parser::SelectorList;
use std::mem;
use style::selector_parser::SelectorParser;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylearc::Arc;
use style::stylesheets::{StyleRule, Origin};

#[dom_struct]
pub struct CSSStyleRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    stylerule: Arc<Locked<StyleRule>>,
    style_decl: MutNullableJS<CSSStyleDeclaration>,
}

impl CSSStyleRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, stylerule: Arc<Locked<StyleRule>>)
                     -> CSSStyleRule {
        CSSStyleRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            stylerule: stylerule,
            style_decl: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               stylerule: Arc<Locked<StyleRule>>) -> Root<CSSStyleRule> {
        reflect_dom_object(box CSSStyleRule::new_inherited(parent_stylesheet, stylerule),
                           window,
                           CSSStyleRuleBinding::Wrap)
    }
}

impl SpecificCSSRule for CSSStyleRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::STYLE_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.stylerule.read_with(&guard).to_css_string(&guard).into()
    }
}

impl CSSStyleRuleMethods for CSSStyleRule {
    // https://drafts.csswg.org/cssom/#dom-cssstylerule-style
    fn Style(&self) -> Root<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let guard = self.cssrule.shared_lock().read();
            CSSStyleDeclaration::new(
                self.global().as_window(),
                CSSStyleOwner::CSSRule(
                    JS::from_ref(self.upcast()),
                    self.stylerule.read_with(&guard).block.clone()
                ),
                None,
                CSSModificationAccess::ReadWrite
            )
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssstylerule-selectortext
    fn SelectorText(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        let stylerule = self.stylerule.read_with(&guard);
        return DOMString::from_string(stylerule.selectors.to_css_string());
    }

    // https://drafts.csswg.org/cssom/#dom-cssstylerule-selectortext
    fn SetSelectorText(&self, value: DOMString) {
        // It's not clear from the spec if we should use the stylesheet's namespaces.
        // https://github.com/w3c/csswg-drafts/issues/1511
        let namespaces = self.cssrule.parent_stylesheet().style_stylesheet().contents.namespaces.read();
        let parser = SelectorParser {
            stylesheet_origin: Origin::Author,
            namespaces: &namespaces,
        };
        let mut css_parser = CssParserInput::new(&*value);
        let mut css_parser = CssParser::new(&mut css_parser);
        if let Ok(mut s) = SelectorList::parse(&parser, &mut css_parser) {
            // This mirrors what we do in CSSStyleOwner::mutate_associated_block.
            let mut guard = self.cssrule.shared_lock().write();
            let mut stylerule = self.stylerule.write_with(&mut guard);
            mem::swap(&mut stylerule.selectors, &mut s);
            // It seems like we will want to avoid having to invalidate all
            // stylesheets eventually!
            self.global().as_window().Document().invalidate_stylesheets();
        }
    }
}
