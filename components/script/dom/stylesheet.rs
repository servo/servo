/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::StyleSheetBinding::StyleSheetMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::element::Element;
use crate::dom::medialist::MediaList;

#[dom_struct]
pub struct StyleSheet {
    reflector_: Reflector,
    type_: DOMString,
    href: Option<DOMString>,
    title: Option<DOMString>,
}

impl StyleSheet {
    #[allow(crown::unrooted_must_root)]
    pub fn new_inherited(
        type_: DOMString,
        href: Option<DOMString>,
        title: Option<DOMString>,
    ) -> StyleSheet {
        StyleSheet {
            reflector_: Reflector::new(),
            type_,
            href,
            title,
        }
    }
}

impl StyleSheetMethods for StyleSheet {
    // https://drafts.csswg.org/cssom/#dom-stylesheet-type
    fn Type_(&self) -> DOMString {
        self.type_.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-href
    fn GetHref(&self) -> Option<DOMString> {
        self.href.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-ownernode
    fn GetOwnerNode(&self) -> Option<DomRoot<Element>> {
        self.downcast::<CSSStyleSheet>().and_then(|s| s.get_owner())
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-media
    fn Media(&self) -> DomRoot<MediaList> {
        self.downcast::<CSSStyleSheet>().unwrap().medialist()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-title
    fn GetTitle(&self) -> Option<DOMString> {
        self.title.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-disabled
    fn Disabled(&self) -> bool {
        self.downcast::<CSSStyleSheet>().unwrap().disabled()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-disabled
    fn SetDisabled(&self, disabled: bool) {
        self.downcast::<CSSStyleSheet>()
            .unwrap()
            .set_disabled(disabled)
    }
}
