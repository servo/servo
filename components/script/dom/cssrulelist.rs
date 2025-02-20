/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::Locked;
use style::stylesheets::{
    AllowImportRules, CssRuleType, CssRuleTypes, CssRules, CssRulesHelpers, KeyframesRule,
    RulesMutateError, StylesheetLoader as StyleStylesheetLoader,
};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CSSRuleListBinding::CSSRuleListMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::csskeyframerule::CSSKeyframeRule;
use crate::dom::cssrule::CSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;
use crate::stylesheet_loader::StylesheetLoader;

unsafe_no_jsmanaged_fields!(RulesSource);

impl From<RulesMutateError> for Error {
    fn from(other: RulesMutateError) -> Self {
        match other {
            RulesMutateError::Syntax => Error::Syntax,
            RulesMutateError::IndexSize => Error::IndexSize,
            RulesMutateError::HierarchyRequest => Error::HierarchyRequest,
            RulesMutateError::InvalidState => Error::InvalidState,
        }
    }
}

#[dom_struct]
pub(crate) struct CSSRuleList {
    reflector_: Reflector,
    parent_stylesheet: Dom<CSSStyleSheet>,
    #[ignore_malloc_size_of = "Arc"]
    rules: RulesSource,
    dom_rules: DomRefCell<Vec<MutNullableDom<CSSRule>>>,
}

pub(crate) enum RulesSource {
    Rules(Arc<Locked<CssRules>>),
    Keyframes(Arc<Locked<KeyframesRule>>),
}

impl CSSRuleList {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        rules: RulesSource,
    ) -> CSSRuleList {
        let guard = parent_stylesheet.shared_lock().read();
        let dom_rules = match rules {
            RulesSource::Rules(ref rules) => rules
                .read_with(&guard)
                .0
                .iter()
                .map(|_| MutNullableDom::new(None))
                .collect(),
            RulesSource::Keyframes(ref rules) => rules
                .read_with(&guard)
                .keyframes
                .iter()
                .map(|_| MutNullableDom::new(None))
                .collect(),
        };

        CSSRuleList {
            reflector_: Reflector::new(),
            parent_stylesheet: Dom::from_ref(parent_stylesheet),
            rules,
            dom_rules: DomRefCell::new(dom_rules),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        rules: RulesSource,
        can_gc: CanGc,
    ) -> DomRoot<CSSRuleList> {
        reflect_dom_object(
            Box::new(CSSRuleList::new_inherited(parent_stylesheet, rules)),
            window,
            can_gc,
        )
    }

    /// Should only be called for CssRules-backed rules. Use append_lazy_rule
    /// for keyframes-backed rules.
    pub(crate) fn insert_rule(
        &self,
        rule: &str,
        idx: u32,
        containing_rule_types: CssRuleTypes,
        parse_relative_rule_type: Option<CssRuleType>,
    ) -> Fallible<u32> {
        let css_rules = if let RulesSource::Rules(ref rules) = self.rules {
            rules
        } else {
            panic!("Called insert_rule on non-CssRule-backed CSSRuleList");
        };

        let global = self.global();
        let window = global.as_window();
        let index = idx as usize;

        let parent_stylesheet = self.parent_stylesheet.style_stylesheet();
        let owner = self
            .parent_stylesheet
            .get_owner()
            .and_then(DomRoot::downcast::<HTMLElement>);
        let loader = owner
            .as_ref()
            .map(|element| StylesheetLoader::for_element(element));
        let new_rule = css_rules.insert_rule(
            &parent_stylesheet.shared_lock,
            rule,
            &parent_stylesheet.contents,
            index,
            containing_rule_types,
            parse_relative_rule_type,
            loader.as_ref().map(|l| l as &dyn StyleStylesheetLoader),
            AllowImportRules::Yes,
        )?;

        let parent_stylesheet = &*self.parent_stylesheet;
        let dom_rule = CSSRule::new_specific(window, parent_stylesheet, new_rule);
        self.dom_rules
            .borrow_mut()
            .insert(index, MutNullableDom::new(Some(&*dom_rule)));
        Ok(idx)
    }

    /// In case of a keyframe rule, index must be valid.
    pub(crate) fn remove_rule(&self, index: u32) -> ErrorResult {
        let index = index as usize;
        let mut guard = self.parent_stylesheet.shared_lock().write();

        match self.rules {
            RulesSource::Rules(ref css_rules) => {
                css_rules.write_with(&mut guard).remove_rule(index)?;
                let mut dom_rules = self.dom_rules.borrow_mut();
                if let Some(r) = dom_rules[index].get() {
                    r.detach()
                }
                dom_rules.remove(index);
                Ok(())
            },
            RulesSource::Keyframes(ref kf) => {
                // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-deleterule
                let mut dom_rules = self.dom_rules.borrow_mut();
                if let Some(r) = dom_rules[index].get() {
                    r.detach()
                }
                dom_rules.remove(index);
                kf.write_with(&mut guard).keyframes.remove(index);
                Ok(())
            },
        }
    }

    /// Remove parent stylesheets from all children
    pub(crate) fn deparent_all(&self) {
        for rule in self.dom_rules.borrow().iter() {
            if let Some(r) = rule.get() {
                DomRoot::upcast(r).deparent()
            }
        }
    }

    pub(crate) fn item(&self, idx: u32) -> Option<DomRoot<CSSRule>> {
        self.dom_rules.borrow().get(idx as usize).map(|rule| {
            rule.or_init(|| {
                let parent_stylesheet = &self.parent_stylesheet;
                let guard = parent_stylesheet.shared_lock().read();
                match self.rules {
                    RulesSource::Rules(ref rules) => CSSRule::new_specific(
                        self.global().as_window(),
                        parent_stylesheet,
                        rules.read_with(&guard).0[idx as usize].clone(),
                    ),
                    RulesSource::Keyframes(ref rules) => DomRoot::upcast(CSSKeyframeRule::new(
                        self.global().as_window(),
                        parent_stylesheet,
                        rules.read_with(&guard).keyframes[idx as usize].clone(),
                        CanGc::note(),
                    )),
                }
            })
        })
    }

    /// Add a rule to the list of DOM rules. This list is lazy,
    /// so we just append a placeholder.
    ///
    /// Should only be called for keyframes-backed rules, use insert_rule
    /// for CssRules-backed rules
    pub(crate) fn append_lazy_dom_rule(&self) {
        if let RulesSource::Rules(..) = self.rules {
            panic!("Can only call append_lazy_rule with keyframes-backed CSSRules");
        }
        self.dom_rules.borrow_mut().push(MutNullableDom::new(None));
    }
}

impl CSSRuleListMethods<crate::DomTypeHolder> for CSSRuleList {
    // https://drafts.csswg.org/cssom/#ref-for-dom-cssrulelist-item-1
    fn Item(&self, idx: u32) -> Option<DomRoot<CSSRule>> {
        self.item(idx)
    }

    // https://drafts.csswg.org/cssom/#dom-cssrulelist-length
    fn Length(&self) -> u32 {
        self.dom_rules.borrow().len() as u32
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<CSSRule>> {
        self.Item(index)
    }
}
