/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StyleSheetBinding;
use dom::bindings::codegen::Bindings::StyleSheetBinding::StyleSheetMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct StyleSheet<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    type_: DOMString,
    href: Option<DOMString>,
    title: Option<DOMString>,
}

impl<TH: TypeHolderTrait> StyleSheet<TH> {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(type_: DOMString,
                         href: Option<DOMString>,
                         title: Option<DOMString>) -> StyleSheet<TH> {
        StyleSheet {
            reflector_: Reflector::new(),
            type_: type_,
            href: href,
            title: title,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, type_: DOMString,
               href: Option<DOMString>,
               title: Option<DOMString>) -> DomRoot<StyleSheet<TH>> {
        reflect_dom_object(Box::new(StyleSheet::new_inherited(type_, href, title)),
                           window,
                           StyleSheetBinding::Wrap)
    }
}


impl<TH: TypeHolderTrait> StyleSheetMethods for StyleSheet<TH> {
    // https://drafts.csswg.org/cssom/#dom-stylesheet-type
    fn Type_(&self) -> DOMString {
        self.type_.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-href
    fn GetHref(&self) -> Option<DOMString> {
        self.href.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-title
    fn GetTitle(&self) -> Option<DOMString> {
        self.title.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-disabled
    fn Disabled(&self) -> bool {
        self.downcast::<CSSStyleSheet<TH>>().unwrap().disabled()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-disabled
    fn SetDisabled(&self, disabled: bool) {
        self.downcast::<CSSStyleSheet<TH>>().unwrap().set_disabled(disabled)
    }
}
