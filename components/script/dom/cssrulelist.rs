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
use dom::cssrule::CSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use style::parser::ParserContextExtraData;
use style::stylesheets::{CssRules, Origin};
use style::stylesheets::CssRule as StyleCssRule;

no_jsmanaged_fields!(CssRules);

#[dom_struct]
pub struct CSSRuleList {
    reflector_: Reflector,
    sheet: JS<CSSStyleSheet>,
    #[ignore_heap_size_of = "Arc"]
    rules: CssRules,
    dom_rules: DOMRefCell<Vec<MutNullableHeap<JS<CSSRule>>>>
}

impl CSSRuleList {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(sheet: &CSSStyleSheet, rules: CssRules) -> CSSRuleList {
        let dom_rules = rules.0.read().iter().map(|_| MutNullableHeap::new(None)).collect();
        CSSRuleList {
            reflector_: Reflector::new(),
            sheet: JS::from_ref(sheet),
            rules: rules,
            dom_rules: DOMRefCell::new(dom_rules),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, sheet: &CSSStyleSheet, rules: CssRules) -> Root<CSSRuleList> {
        reflect_dom_object(box CSSRuleList::new_inherited(sheet, rules),
                           window,
                           CSSRuleListBinding::Wrap)
    }

    // https://drafts.csswg.org/cssom/#insert-a-css-rule
    pub fn insert_rule(&self, rule: &str, idx: u32) -> Fallible<u32> {
        /// Insert an item into a vector, appending if it is out of bounds
        fn insert<T>(vec: &mut Vec<T>, index: usize, item: T) {
            if index >= vec.len() {
                vec.push(item);
            } else {
                vec.insert(index, item);
            }
        }
        let global = self.global();
        let window = global.as_window();
        let doc = window.Document();
        let index = idx as usize;

        // Step 1, 2
        // XXXManishearth get url from correct location
        // XXXManishearth should we also store the namespace map?
        let new_rule = try!(StyleCssRule::from_str(&rule, Origin::Author,
                                                   doc.url().clone(),
                                                   ParserContextExtraData::default())
                          .map_err(|_| Error::Syntax));

        {
            let rules = self.rules.0.read();
            // Step 3, 4
            if index > rules.len() {
                return Err(Error::IndexSize);
            }

            // XXXManishearth Step 5 (throw HierarchyRequestError in invalid situations)

            // Step 6
            if let StyleCssRule::Namespace(..) = new_rule {
                if !CssRules::only_ns_or_import(&rules) {
                    return Err(Error::InvalidState);
                }
            }
        }

        insert(&mut self.rules.0.write(), index, new_rule.clone());
        let dom_rule = CSSRule::new_specific(&window, &self.sheet, new_rule);
        insert(&mut self.dom_rules.borrow_mut(),
               index, MutNullableHeap::new(Some(&*dom_rule)));
        Ok((idx))
    }

    // https://drafts.csswg.org/cssom/#remove-a-css-rule
    pub fn remove_rule(&self, index: u32) -> ErrorResult {
        let index = index as usize;

        {
            let rules = self.rules.0.read();
            if index >= rules.len() {
                return Err(Error::IndexSize);
            }
            let ref rule = rules[index];
            if let StyleCssRule::Namespace(..) = *rule {
                if !CssRules::only_ns_or_import(&rules) {
                    return Err(Error::InvalidState);
                }
            }
        }

        let mut dom_rules = self.dom_rules.borrow_mut();
        self.rules.0.write().remove(index);
        dom_rules[index].get().map(|r| r.disown());
        dom_rules.remove(index);
        Ok(())
    }
}

impl CSSRuleListMethods for CSSRuleList {
    // https://drafts.csswg.org/cssom/#ref-for-dom-cssrulelist-item-1
    fn Item(&self, idx: u32) -> Option<Root<CSSRule>> {
        self.dom_rules.borrow().get(idx as usize).map(|rule| {
            rule.or_init(|| {
                CSSRule::new_specific(self.global().as_window(),
                                     &self.sheet,
                                     self.rules.0.read()[idx as usize].clone())
            })
        })
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

