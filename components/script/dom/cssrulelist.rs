/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CSSRuleListBinding;
use dom::bindings::codegen::Bindings::CSSRuleListBinding::CSSRuleListMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::csskeyframerule::CSSKeyframeRule;
use dom::cssrule::CSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use style::shared_lock::Locked;
use style::stylearc::Arc;
use style::stylesheets::{CssRules, CssRulesHelpers, KeyframesRule, RulesMutateError};

#[allow(unsafe_code)]
unsafe_no_jsmanaged_fields!(RulesSource);

unsafe_no_jsmanaged_fields!(CssRules);

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
pub struct CSSRuleList {
    reflector_: Reflector,
    parent_stylesheet: JS<CSSStyleSheet>,
    #[ignore_heap_size_of = "Arc"]
    rules: RulesSource,
    dom_rules: DOMRefCell<Vec<MutNullableJS<CSSRule>>>
}

pub enum RulesSource {
    Rules(Arc<Locked<CssRules>>),
    Keyframes(Arc<Locked<KeyframesRule>>),
}

impl CSSRuleList {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(parent_stylesheet: &CSSStyleSheet, rules: RulesSource) -> CSSRuleList {
        let guard = parent_stylesheet.shared_lock().read();
        let dom_rules = match rules {
            RulesSource::Rules(ref rules) => {
                rules.read_with(&guard).0.iter().map(|_| MutNullableJS::new(None)).collect()
            }
            RulesSource::Keyframes(ref rules) => {
                rules.read_with(&guard).keyframes.iter().map(|_| MutNullableJS::new(None)).collect()
            }
        };

        CSSRuleList {
            reflector_: Reflector::new(),
            parent_stylesheet: JS::from_ref(parent_stylesheet),
            rules: rules,
            dom_rules: DOMRefCell::new(dom_rules),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               rules: RulesSource) -> Root<CSSRuleList> {
        reflect_dom_object(box CSSRuleList::new_inherited(parent_stylesheet, rules),
                           window,
                           CSSRuleListBinding::Wrap)
    }

    /// Should only be called for CssRules-backed rules. Use append_lazy_rule
    /// for keyframes-backed rules.
    pub fn insert_rule(&self, rule: &str, idx: u32, nested: bool) -> Fallible<u32> {
        let css_rules = if let RulesSource::Rules(ref rules) = self.rules {
            rules
        } else {
            panic!("Called insert_rule on non-CssRule-backed CSSRuleList");
        };

        let global = self.global();
        let window = global.as_window();
        let index = idx as usize;

        let parent_stylesheet = self.parent_stylesheet.style_stylesheet();
        let new_rule =
            css_rules.insert_rule(&parent_stylesheet.shared_lock,
                                  rule,
                                  &parent_stylesheet.contents,
                                  index,
                                  nested,
                                  None)?;

        let parent_stylesheet = &*self.parent_stylesheet;
        let dom_rule = CSSRule::new_specific(&window, parent_stylesheet, new_rule);
        self.dom_rules.borrow_mut().insert(index, MutNullableJS::new(Some(&*dom_rule)));
        Ok((idx))
    }

    // In case of a keyframe rule, index must be valid.
    pub fn remove_rule(&self, index: u32) -> ErrorResult {
        let index = index as usize;
        let mut guard = self.parent_stylesheet.shared_lock().write();

        match self.rules {
            RulesSource::Rules(ref css_rules) => {
                css_rules.write_with(&mut guard).remove_rule(index)?;
                let mut dom_rules = self.dom_rules.borrow_mut();
                dom_rules[index].get().map(|r| r.detach());
                dom_rules.remove(index);
                Ok(())
            }
            RulesSource::Keyframes(ref kf) => {
                // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-deleterule
                let mut dom_rules = self.dom_rules.borrow_mut();
                dom_rules[index].get().map(|r| r.detach());
                dom_rules.remove(index);
                kf.write_with(&mut guard).keyframes.remove(index);
                Ok(())
            }
        }
    }

    // Remove parent stylesheets from all children
    pub fn deparent_all(&self) {
        for rule in self.dom_rules.borrow().iter() {
            rule.get().map(|r| Root::upcast(r).deparent());
        }
    }

    pub fn item(&self, idx: u32) -> Option<Root<CSSRule>> {
        self.dom_rules.borrow().get(idx as usize).map(|rule| {
            rule.or_init(|| {
                let parent_stylesheet = &self.parent_stylesheet;
                let guard = parent_stylesheet.shared_lock().read();
                match self.rules {
                    RulesSource::Rules(ref rules) => {
                        CSSRule::new_specific(self.global().as_window(),
                                             parent_stylesheet,
                                             rules.read_with(&guard).0[idx as usize].clone())
                    }
                    RulesSource::Keyframes(ref rules) => {
                        Root::upcast(CSSKeyframeRule::new(self.global().as_window(),
                                                          parent_stylesheet,
                                                          rules.read_with(&guard)
                                                                .keyframes[idx as usize]
                                                                .clone()))
                    }
                }

            })
        })
    }

    /// Add a rule to the list of DOM rules. This list is lazy,
    /// so we just append a placeholder.
    ///
    /// Should only be called for keyframes-backed rules, use insert_rule
    /// for CssRules-backed rules
    pub fn append_lazy_dom_rule(&self) {
        if let RulesSource::Rules(..) = self.rules {
            panic!("Can only call append_lazy_rule with keyframes-backed CSSRules");
        }
        self.dom_rules.borrow_mut().push(MutNullableJS::new(None));
    }
}

impl CSSRuleListMethods for CSSRuleList {
    // https://drafts.csswg.org/cssom/#ref-for-dom-cssrulelist-item-1
    fn Item(&self, idx: u32) -> Option<Root<CSSRule>> {
        self.item(idx)
    }

    // https://drafts.csswg.org/cssom/#dom-cssrulelist-length
    fn Length(&self) -> u32 {
        self.dom_rules.borrow().len() as u32
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<Root<CSSRule>> {
        self.Item(index)
    }
}

