/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::mem;

use cssparser::{Parser as CssParser, ParserInput as CssParserInput, ToCss};
use dom_struct::dom_struct;
use selectors::parser::{ParseRelative, SelectorList};
use servo_arc::Arc;
use style::selector_parser::SelectorParser;
use style::shared_lock::{Locked, SharedRwLockReadGuard, ToCssWithGuard};
use style::stylesheets::{CssRuleType, CssRules, Origin, StyleRule, StylesheetInDocument};

use super::cssgroupingrule::CSSGroupingRule;
use super::cssrule::SpecificCSSRule;
use super::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSStyleRuleBinding::CSSStyleRuleMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSStyleRule {
    css_grouping_rule: CSSGroupingRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    style_rule: RefCell<Arc<Locked<StyleRule>>>,
    style_declaration: MutNullableDom<CSSStyleDeclaration>,
}

impl CSSStyleRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        stylerule: Arc<Locked<StyleRule>>,
    ) -> CSSStyleRule {
        CSSStyleRule {
            css_grouping_rule: CSSGroupingRule::new_inherited(parent_stylesheet),
            style_rule: RefCell::new(stylerule),
            style_declaration: Default::default(),
        }
    }

    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        stylerule: Arc<Locked<StyleRule>>,
        can_gc: CanGc,
    ) -> DomRoot<CSSStyleRule> {
        reflect_dom_object(
            Box::new(CSSStyleRule::new_inherited(parent_stylesheet, stylerule)),
            window,
            can_gc,
        )
    }

    pub(crate) fn ensure_rules(&self) -> Arc<Locked<CssRules>> {
        let lock = self.css_grouping_rule.shared_lock();
        let mut guard = lock.write();
        self.style_rule
            .borrow()
            .write_with(&mut guard)
            .rules
            .get_or_insert_with(|| CssRules::new(vec![], lock))
            .clone()
    }

    pub(crate) fn update_rule(
        &self,
        stylerule: Arc<Locked<StyleRule>>,
        guard: &SharedRwLockReadGuard,
    ) {
        if let Some(ref rules) = stylerule.read_with(guard).rules {
            self.css_grouping_rule.update_rules(rules, guard);
        }

        if let Some(ref style_decl) = self.style_declaration.get() {
            style_decl.update_property_declaration_block(&stylerule.read_with(guard).block);
        }

        *self.style_rule.borrow_mut() = stylerule;
    }
}

impl SpecificCSSRule for CSSStyleRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Style
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_grouping_rule.shared_lock().read();
        self.style_rule
            .borrow()
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}

impl CSSStyleRuleMethods<crate::DomTypeHolder> for CSSStyleRule {
    /// <https://drafts.csswg.org/cssom/#dom-cssstylerule-style>
    fn Style(&self, can_gc: CanGc) -> DomRoot<CSSStyleDeclaration> {
        self.style_declaration.or_init(|| {
            let guard = self.css_grouping_rule.shared_lock().read();
            CSSStyleDeclaration::new(
                self.global().as_window(),
                CSSStyleOwner::CSSRule(
                    Dom::from_ref(self.upcast()),
                    RefCell::new(self.style_rule.borrow().read_with(&guard).block.clone()),
                ),
                None,
                CSSModificationAccess::ReadWrite,
                can_gc,
            )
        })
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstylerule-selectortext>
    fn SelectorText(&self) -> DOMString {
        let guard = self.css_grouping_rule.shared_lock().read();
        DOMString::from_string(
            self.style_rule
                .borrow()
                .read_with(&guard)
                .selectors
                .to_css_string(),
        )
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstylerule-selectortext>
    fn SetSelectorText(&self, value: DOMString) {
        let value = value.str();
        let Ok(mut selector) = ({
            let guard = self.css_grouping_rule.shared_lock().read();
            let sheet = self
                .css_grouping_rule
                .parent_stylesheet()
                .style_stylesheet();
            let contents = sheet.contents(&guard);
            // It's not clear from the spec if we should use the stylesheet's namespaces.
            // https://github.com/w3c/csswg-drafts/issues/1511
            let parser = SelectorParser {
                stylesheet_origin: Origin::Author,
                namespaces: &contents.namespaces,
                url_data: &contents.url_data,
                for_supports_rule: false,
            };
            let mut css_parser = CssParserInput::new(&value);
            let mut css_parser = CssParser::new(&mut css_parser);
            // TODO: Maybe allow setting relative selectors from the OM, if we're in a nested style
            // rule?
            SelectorList::parse(&parser, &mut css_parser, ParseRelative::No)
        }) else {
            return;
        };
        self.css_grouping_rule.parent_stylesheet().will_modify();
        // This mirrors what we do in CSSStyleOwner::mutate_associated_block.
        let mut guard = self.css_grouping_rule.shared_lock().write();
        mem::swap(
            &mut self.style_rule.borrow().write_with(&mut guard).selectors,
            &mut selector,
        );
        self.css_grouping_rule
            .parent_stylesheet()
            .notify_invalidations();
    }
}
