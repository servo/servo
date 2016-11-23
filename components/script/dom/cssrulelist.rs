/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CSSRuleListBinding;
use dom::bindings::codegen::Bindings::CSSRuleListBinding::CSSRuleListMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::csskeyframerule::CSSKeyframeRule;
use dom::cssrule::CSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::parser::ParserContextExtraData;
use style::stylesheets::{CssRules, KeyframesRule, Origin};
use style::stylesheets::CssRule as StyleCssRule;

no_jsmanaged_fields!(RulesSource);
no_jsmanaged_fields!(CssRules);

#[dom_struct]
pub struct CSSRuleList {
    reflector_: Reflector,
    sheet: MutNullableHeap<JS<CSSStyleSheet>>,
    #[ignore_heap_size_of = "Arc"]
    rules: RulesSource,
    dom_rules: DOMRefCell<Vec<MutNullableHeap<JS<CSSRule>>>>
}

pub enum RulesSource {
    Rules(CssRules),
    Keyframes(Arc<RwLock<KeyframesRule>>),
}

impl CSSRuleList {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(sheet: Option<&CSSStyleSheet>, rules: RulesSource) -> CSSRuleList {
        let dom_rules = match rules {
            RulesSource::Rules(ref rules) => {
                rules.0.read().iter().map(|_| MutNullableHeap::new(None)).collect()
            }
            RulesSource::Keyframes(ref rules) => {
                rules.read().keyframes.iter().map(|_| MutNullableHeap::new(None)).collect()
            }
        };

        CSSRuleList {
            reflector_: Reflector::new(),
            sheet: MutNullableHeap::new(sheet),
            rules: rules,
            dom_rules: DOMRefCell::new(dom_rules),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, sheet: Option<&CSSStyleSheet>,
               rules: RulesSource) -> Root<CSSRuleList> {
        reflect_dom_object(box CSSRuleList::new_inherited(sheet, rules),
                           window,
                           CSSRuleListBinding::Wrap)
    }

    /// https://drafts.csswg.org/cssom/#insert-a-css-rule
    ///
    /// Should only be called for CssRules-backed rules. Use append_lazy_rule
    /// for keyframes-backed rules.
    pub fn insert_rule(&self, rule: &str, idx: u32, nested: bool) -> Fallible<u32> {
        use style::stylesheets::SingleRuleParseError;
        /// Insert an item into a vector, appending if it is out of bounds
        fn insert<T>(vec: &mut Vec<T>, index: usize, item: T) {
            if index >= vec.len() {
                vec.push(item);
            } else {
                vec.insert(index, item);
            }
        }

        let css_rules = if let RulesSource::Rules(ref rules) = self.rules {
            rules
        } else {
            panic!("Called insert_rule on non-CssRule-backed CSSRuleList");
        };

        let global = self.global();
        let window = global.as_window();
        let doc = window.Document();
        let index = idx as usize;


        let new_rule = {
            let rules = css_rules.0.read();
            let state = if nested {
                None
            } else {
                Some(CssRules::state_at_index(&rules, index))
            };

            let rev_state = CssRules::state_at_index_rev(&rules, index);

            // Step 1, 2
            // XXXManishearth get url from correct location
            // XXXManishearth should we also store the namespace map?
            let parse_result = StyleCssRule::parse(&rule, Origin::Author,
                                               doc.url().clone(),
                                               ParserContextExtraData::default(),
                                               state);

            if let Err(SingleRuleParseError::Syntax) = parse_result {
                return Err(Error::Syntax)
            }

            // Step 3, 4
            if index > rules.len() {
                return Err(Error::IndexSize);
            }

            let (new_rule, new_state) = try!(parse_result.map_err(|_| Error::HierarchyRequest));

            if new_state > rev_state {
                // We inserted a rule too early, e.g. inserting
                // a regular style rule before @namespace rules
                return Err((Error::HierarchyRequest));
            }

            // Step 6
            if let StyleCssRule::Namespace(..) = new_rule {
                if !CssRules::only_ns_or_import(&rules) {
                    return Err(Error::InvalidState);
                }
            }

            new_rule
        };

        insert(&mut css_rules.0.write(), index, new_rule.clone());
        let sheet = self.sheet.get();
        let sheet = sheet.as_ref().map(|sheet| &**sheet);
        let dom_rule = CSSRule::new_specific(&window, sheet, new_rule);
        insert(&mut self.dom_rules.borrow_mut(),
               index, MutNullableHeap::new(Some(&*dom_rule)));
        Ok((idx))
    }

    // https://drafts.csswg.org/cssom/#remove-a-css-rule
    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-deleterule
    // In case of a keyframe rule, index must be valid.
    pub fn remove_rule(&self, index: u32) -> ErrorResult {
        let index = index as usize;

        match self.rules {
            RulesSource::Rules(ref css_rules) => {
                // https://drafts.csswg.org/cssom/#remove-a-css-rule
                {
                    let rules = css_rules.0.read();

                    // Step 1, 2
                    if index >= rules.len() {
                        return Err(Error::IndexSize);
                    }

                    // Step 3
                    let ref rule = rules[index];

                    // Step 4
                    if let StyleCssRule::Namespace(..) = *rule {
                        if !CssRules::only_ns_or_import(&rules) {
                            return Err(Error::InvalidState);
                        }
                    }
                }

                // Step 5, 6
                let mut dom_rules = self.dom_rules.borrow_mut();
                css_rules.0.write().remove(index);
                dom_rules[index].get().map(|r| r.detach());
                dom_rules.remove(index);
                Ok(())
            }
            RulesSource::Keyframes(ref kf) => {
                // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-deleterule
                let mut dom_rules = self.dom_rules.borrow_mut();
                dom_rules[index].get().map(|r| r.detach());
                dom_rules.remove(index);
                kf.write().keyframes.remove(index);
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
                let sheet = self.sheet.get();
                let sheet = sheet.as_ref().map(|sheet| &**sheet);
                match self.rules {
                    RulesSource::Rules(ref rules) => {
                        CSSRule::new_specific(self.global().as_window(),
                                             sheet,
                                             rules.0.read()[idx as usize].clone())
                    }
                    RulesSource::Keyframes(ref rules) => {
                        Root::upcast(CSSKeyframeRule::new(self.global().as_window(),
                                                          sheet,
                                                          rules.read()
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
        self.dom_rules.borrow_mut().push(MutNullableHeap::new(None));
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

