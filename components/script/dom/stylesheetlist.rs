/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::stylesheets::Stylesheet;

use crate::dom::bindings::codegen::Bindings::StyleSheetListBinding::StyleSheetListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::stylesheet::StyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum StyleSheetListOwner {
    Document(Dom<Document>),
    ShadowRoot(Dom<ShadowRoot>),
}

impl StyleSheetListOwner {
    pub(crate) fn stylesheet_count(&self) -> usize {
        match *self {
            StyleSheetListOwner::Document(ref doc) => doc.stylesheet_count(),
            StyleSheetListOwner::ShadowRoot(ref shadow_root) => shadow_root.stylesheet_count(),
        }
    }

    pub(crate) fn stylesheet_at(&self, index: usize) -> Option<DomRoot<CSSStyleSheet>> {
        match *self {
            StyleSheetListOwner::Document(ref doc) => doc.stylesheet_at(index),
            StyleSheetListOwner::ShadowRoot(ref shadow_root) => shadow_root.stylesheet_at(index),
        }
    }

    pub(crate) fn add_stylesheet(&self, owner: &Element, sheet: Arc<Stylesheet>) {
        match *self {
            StyleSheetListOwner::Document(ref doc) => doc.add_stylesheet(owner, sheet),
            StyleSheetListOwner::ShadowRoot(ref shadow_root) => {
                shadow_root.add_stylesheet(owner, sheet)
            },
        }
    }

    pub(crate) fn remove_stylesheet(&self, owner: &Element, s: &Arc<Stylesheet>) {
        match *self {
            StyleSheetListOwner::Document(ref doc) => doc.remove_stylesheet(owner, s),
            StyleSheetListOwner::ShadowRoot(ref shadow_root) => {
                shadow_root.remove_stylesheet(owner, s)
            },
        }
    }

    pub(crate) fn invalidate_stylesheets(&self) {
        match *self {
            StyleSheetListOwner::Document(ref doc) => doc.invalidate_stylesheets(),
            StyleSheetListOwner::ShadowRoot(ref shadow_root) => {
                shadow_root.invalidate_stylesheets()
            },
        }
    }
}

#[dom_struct]
pub(crate) struct StyleSheetList {
    reflector_: Reflector,
    document_or_shadow_root: StyleSheetListOwner,
}

impl StyleSheetList {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(doc_or_sr: StyleSheetListOwner) -> StyleSheetList {
        StyleSheetList {
            reflector_: Reflector::new(),
            document_or_shadow_root: doc_or_sr,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(window: &Window, doc_or_sr: StyleSheetListOwner) -> DomRoot<StyleSheetList> {
        reflect_dom_object(
            Box::new(StyleSheetList::new_inherited(doc_or_sr)),
            window,
            CanGc::note(),
        )
    }
}

impl StyleSheetListMethods<crate::DomTypeHolder> for StyleSheetList {
    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-length
    fn Length(&self) -> u32 {
        self.document_or_shadow_root.stylesheet_count() as u32
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-item
    fn Item(&self, index: u32) -> Option<DomRoot<StyleSheet>> {
        // XXXManishearth this  doesn't handle the origin clean flag and is a
        // cors vulnerability
        self.document_or_shadow_root
            .stylesheet_at(index as usize)
            .map(DomRoot::upcast)
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<StyleSheet>> {
        self.Item(index)
    }
}
