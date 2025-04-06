/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{CssRuleTypes, Stylesheet as StyleStyleSheet};

use crate::dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use crate::dom::bindings::codegen::GenericBindings::CSSRuleListBinding::CSSRuleList_Binding::CSSRuleListMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrulelist::{CSSRuleList, RulesSource};
use crate::dom::element::Element;
use crate::dom::medialist::MediaList;
use crate::dom::node::NodeTraits;
use crate::dom::stylesheet::StyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSStyleSheet {
    stylesheet: StyleSheet,
    owner: MutNullableDom<Element>,
    rulelist: MutNullableDom<CSSRuleList>,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    style_stylesheet: Arc<StyleStyleSheet>,
    origin_clean: Cell<bool>,
}

impl CSSStyleSheet {
    fn new_inherited(
        owner: &Element,
        type_: DOMString,
        href: Option<DOMString>,
        title: Option<DOMString>,
        stylesheet: Arc<StyleStyleSheet>,
    ) -> CSSStyleSheet {
        CSSStyleSheet {
            stylesheet: StyleSheet::new_inherited(type_, href, title),
            owner: MutNullableDom::new(Some(owner)),
            rulelist: MutNullableDom::new(None),
            style_stylesheet: stylesheet,
            origin_clean: Cell::new(true),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        owner: &Element,
        type_: DOMString,
        href: Option<DOMString>,
        title: Option<DOMString>,
        stylesheet: Arc<StyleStyleSheet>,
        can_gc: CanGc,
    ) -> DomRoot<CSSStyleSheet> {
        reflect_dom_object(
            Box::new(CSSStyleSheet::new_inherited(
                owner, type_, href, title, stylesheet,
            )),
            window,
            can_gc,
        )
    }

    fn rulelist(&self, can_gc: CanGc) -> DomRoot<CSSRuleList> {
        self.rulelist.or_init(|| {
            let rules = self.style_stylesheet.contents.rules.clone();
            CSSRuleList::new(
                self.global().as_window(),
                self,
                RulesSource::Rules(rules),
                can_gc,
            )
        })
    }

    pub(crate) fn disabled(&self) -> bool {
        self.style_stylesheet.disabled()
    }

    pub(crate) fn get_owner(&self) -> Option<DomRoot<Element>> {
        self.owner.get()
    }

    pub(crate) fn set_disabled(&self, disabled: bool) {
        if self.style_stylesheet.set_disabled(disabled) && self.get_owner().is_some() {
            self.get_owner()
                .unwrap()
                .stylesheet_list_owner()
                .invalidate_stylesheets();
        }
    }

    pub(crate) fn set_owner(&self, value: Option<&Element>) {
        self.owner.set(value);
    }

    pub(crate) fn shared_lock(&self) -> &SharedRwLock {
        &self.style_stylesheet.shared_lock
    }

    pub(crate) fn style_stylesheet(&self) -> &StyleStyleSheet {
        &self.style_stylesheet
    }

    pub(crate) fn set_origin_clean(&self, origin_clean: bool) {
        self.origin_clean.set(origin_clean);
    }

    pub(crate) fn medialist(&self, can_gc: CanGc) -> DomRoot<MediaList> {
        MediaList::new(
            self.global().as_window(),
            self,
            self.style_stylesheet().media.clone(),
            can_gc,
        )
    }
}

impl CSSStyleSheetMethods<crate::DomTypeHolder> for CSSStyleSheet {
    /// <https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules>
    fn GetCssRules(&self, can_gc: CanGc) -> Fallible<DomRoot<CSSRuleList>> {
        if !self.origin_clean.get() {
            return Err(Error::Security);
        }
        Ok(self.rulelist(can_gc))
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstylesheet-insertrule>
    fn InsertRule(&self, rule: DOMString, index: u32, can_gc: CanGc) -> Fallible<u32> {
        if !self.origin_clean.get() {
            return Err(Error::Security);
        }
        self.rulelist(can_gc)
            .insert_rule(&rule, index, CssRuleTypes::default(), None, can_gc)
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstylesheet-deleterule>
    fn DeleteRule(&self, index: u32, can_gc: CanGc) -> ErrorResult {
        if !self.origin_clean.get() {
            return Err(Error::Security);
        }
        self.rulelist(can_gc).remove_rule(index)
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstylesheet-rules>
    fn GetRules(&self, can_gc: CanGc) -> Fallible<DomRoot<CSSRuleList>> {
        self.GetCssRules(can_gc)
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstylesheet-removerule>
    fn RemoveRule(&self, index: u32, can_gc: CanGc) -> ErrorResult {
        self.DeleteRule(index, can_gc)
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstylesheet-addrule>
    fn AddRule(
        &self,
        selector: DOMString,
        block: DOMString,
        optional_index: Option<u32>,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        // > 1. Let *rule* be an empty string.
        // > 2. Append *selector* to *rule*.
        let mut rule = selector;

        // > 3. Append " { " to *rule*.
        // > 4. If *block* is not empty, append *block*, followed by a space, to *rule*.
        // > 5. Append "}" to *rule*.
        if block.is_empty() {
            rule.push_str(" { }");
        } else {
            rule.push_str(" { ");
            rule.push_str(block.str());
            rule.push_str(" } ");
        };

        // > 6. Let *index* be *optionalIndex* if provided, or the number of CSS rules in the stylesheet otherwise.
        let index = optional_index.unwrap_or_else(|| self.rulelist(can_gc).Length());

        // > 7. Call `insertRule()`, with *rule* and *index* as arguments.
        self.InsertRule(rule, index, can_gc)?;

        // > 8. Return -1.
        Ok(-1)
    }
}
