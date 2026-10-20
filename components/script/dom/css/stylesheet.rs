/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use script_bindings::domstring::TracedDOMString;
use script_bindings::reflector::Reflector;
use xml5ever::local_name;

use super::cssstylesheet::CSSStyleSheet;
use crate::dom::Node;
use crate::dom::bindings::codegen::Bindings::StyleSheetBinding::StyleSheetMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::medialist::MediaList;

#[dom_struct]
pub(crate) struct StyleSheet {
    reflector_: Reflector,
    type_: TracedDOMString,
    href: Option<TracedDOMString>,
}

impl StyleSheet {
    pub(crate) fn new_inherited(type_: DOMString, href: Option<DOMString>) -> StyleSheet {
        StyleSheet {
            reflector_: Reflector::new(),
            type_: type_.traced(),
            href: href.map(|domstring| domstring.traced()),
        }
    }
}

impl StyleSheetMethods<crate::DomTypeHolder> for StyleSheet {
    /// <https://drafts.csswg.org/cssom/#dom-stylesheet-type>
    fn Type(&self) -> DOMString {
        self.type_.clone().root()
    }

    /// <https://drafts.csswg.org/cssom/#dom-stylesheet-href>
    fn GetHref(&self) -> Option<DOMString> {
        self.href.clone().map(|domstring| domstring.root())
    }

    /// <https://drafts.csswg.org/cssom/#dom-stylesheet-ownernode>
    fn GetOwnerNode(&self) -> Option<DomRoot<Element>> {
        self.downcast::<CSSStyleSheet>()
            .and_then(|s| s.owner_node())
    }

    /// <https://drafts.csswg.org/cssom/#dom-stylesheet-media>
    fn Media(&self, cx: &mut JSContext) -> DomRoot<MediaList> {
        self.downcast::<CSSStyleSheet>().unwrap().medialist(cx)
    }

    /// <https://drafts.csswg.org/cssom/#dom-stylesheet-title>
    fn GetTitle(&self) -> Option<DOMString> {
        let css_style_sheet = self.downcast::<CSSStyleSheet>()?;
        let owner_element = css_style_sheet.owner_node()?;
        if !owner_element.upcast::<Node>().is_in_a_document_tree() {
            return None;
        }

        let title_string = owner_element.get_attribute_string_value(&local_name!("title"))?;
        if title_string.is_empty() {
            return None;
        }
        Some(title_string.into())
    }

    /// <https://drafts.csswg.org/cssom/#dom-stylesheet-disabled>
    fn Disabled(&self) -> bool {
        self.downcast::<CSSStyleSheet>().unwrap().disabled()
    }

    /// <https://drafts.csswg.org/cssom/#dom-stylesheet-disabled>
    fn SetDisabled(&self, no_gc: &NoGC, disabled: bool) {
        self.downcast::<CSSStyleSheet>()
            .unwrap()
            .set_disabled(no_gc, disabled)
    }
}
