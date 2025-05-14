/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_arc::Arc;
use style::media_queries::MediaList as StyleMediaList;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{
    AllowImportRules, CssRuleTypes, Origin, Stylesheet as StyleStyleSheet, UrlExtraData,
};

use crate::dom::bindings::codegen::Bindings::CSSStyleSheetBinding::{
    CSSStyleSheetInit, CSSStyleSheetMethods,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::GenericBindings::CSSRuleListBinding::CSSRuleList_Binding::CSSRuleListMethods;
use crate::dom::bindings::codegen::UnionTypes::MediaListOrString;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::{
    DomGlobal, reflect_dom_object, reflect_dom_object_with_proto,
};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
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
    is_constructed: bool,
}

impl CSSStyleSheet {
    fn new_inherited(
        owner: Option<&Element>,
        type_: DOMString,
        href: Option<DOMString>,
        title: Option<DOMString>,
        stylesheet: Arc<StyleStyleSheet>,
        is_constructed: bool,
    ) -> CSSStyleSheet {
        CSSStyleSheet {
            stylesheet: StyleSheet::new_inherited(type_, href, title),
            owner: MutNullableDom::new(owner),
            rulelist: MutNullableDom::new(None),
            style_stylesheet: stylesheet,
            origin_clean: Cell::new(true),
            is_constructed,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        owner: Option<&Element>,
        type_: DOMString,
        href: Option<DOMString>,
        title: Option<DOMString>,
        stylesheet: Arc<StyleStyleSheet>,
        is_constructed: bool,
        can_gc: CanGc,
    ) -> DomRoot<CSSStyleSheet> {
        reflect_dom_object(
            Box::new(CSSStyleSheet::new_inherited(
                owner,
                type_,
                href,
                title,
                stylesheet,
                is_constructed,
            )),
            window,
            can_gc,
        )
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        owner: Option<&Element>,
        type_: DOMString,
        href: Option<DOMString>,
        title: Option<DOMString>,
        stylesheet: Arc<StyleStyleSheet>,
        is_constructed: bool,
        can_gc: CanGc,
    ) -> DomRoot<CSSStyleSheet> {
        reflect_dom_object_with_proto(
            Box::new(CSSStyleSheet::new_inherited(
                owner,
                type_,
                href,
                title,
                stylesheet,
                is_constructed,
            )),
            window,
            proto,
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

    /// <https://drafts.csswg.org/cssom/#concept-css-style-sheet-constructed-flag>
    #[inline]
    pub(crate) fn is_constructed(&self) -> bool {
        self.is_constructed
    }
}

impl CSSStyleSheetMethods<crate::DomTypeHolder> for CSSStyleSheet {
    /// <https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssstylesheet>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        options: &CSSStyleSheetInit,
    ) -> DomRoot<Self> {
        let doc = window.Document();
        let shared_lock = doc.style_shared_lock().clone();
        let media = Arc::new(shared_lock.wrap(match &options.media {
            Some(media) => match media {
                MediaListOrString::MediaList(media_list) => media_list.clone_media_list(),
                MediaListOrString::String(str) => MediaList::parse_media_list(str, window),
            },
            None => StyleMediaList::empty(),
        }));
        let stylesheet = Arc::new(StyleStyleSheet::from_str(
            "",
            UrlExtraData(window.get_url().get_arc()),
            Origin::Author,
            media,
            shared_lock,
            None,
            window.css_error_reporter(),
            doc.quirks_mode(),
            AllowImportRules::No,
        ));
        if options.disabled {
            stylesheet.set_disabled(true);
        }
        Self::new_with_proto(
            window,
            proto,
            None, // owner
            "text/css".into(),
            None, // href
            None, // title
            stylesheet,
            true, // is_constructed
            can_gc,
        )
    }

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
            rule.push_str(" }");
        };

        // > 6. Let *index* be *optionalIndex* if provided, or the number of CSS rules in the stylesheet otherwise.
        let index = optional_index.unwrap_or_else(|| self.rulelist(can_gc).Length());

        // > 7. Call `insertRule()`, with *rule* and *index* as arguments.
        self.InsertRule(rule, index, can_gc)?;

        // > 8. Return -1.
        Ok(-1)
    }

    /// <https://drafts.csswg.org/cssom/#synchronously-replace-the-rules-of-a-cssstylesheet>
    fn ReplaceSync(&self, text: USVString) -> Result<(), Error> {
        // Step 1. If the constructed flag is not set throw a NotAllowedError
        if !self.is_constructed {
            return Err(Error::NotAllowed);
        }

        // Step 2. Let rules be the result of running parse a stylesheet’s contents from text.
        let global = self.global();
        let window = global.as_window();

        StyleStyleSheet::update_from_str(
            &self.style_stylesheet,
            &text,
            UrlExtraData(window.get_url().get_arc()),
            None,
            window.css_error_reporter(),
            AllowImportRules::No, // Step 3.If rules contains one or more @import rules, remove those rules from rules.
        );

        // Step 4. Set sheet’s CSS rules to rules.
        // We reset our rule list, which will be initialized properly
        // at the next getter access.
        self.rulelist.set(None);

        Ok(())
    }
}
