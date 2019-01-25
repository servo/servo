/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{self, ShadowRootMode};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documentorshadowroot::{DocumentOrShadowRoot, DocumentOrShadowRootImpl};
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeFlags};
use crate::dom::stylesheetlist::StyleSheetList;
use crate::dom::window::Window;
use dom_struct::dom_struct;

// https://dom.spec.whatwg.org/#interface-shadowroot
#[dom_struct]
pub struct ShadowRoot {
    document_fragment: DocumentFragment,
    document_or_shadow_root: DocumentOrShadowRootImpl,
    has_browsing_context: bool,
    host: Dom<Element>,
    stylesheet_list: MutNullableDom<StyleSheetList>,
    window: Dom<Window>,
}

impl ShadowRoot {
    #[allow(unrooted_must_root)]
    fn new_inherited(host: &Element, document: &Document) -> ShadowRoot {
        let has_browsing_context = true;
        let document_fragment = DocumentFragment::new_inherited(document);
        document_fragment
            .upcast::<Node>()
            .set_flag(NodeFlags::IS_IN_SHADOW_TREE, true);
        ShadowRoot {
            document_fragment,
            document_or_shadow_root: DocumentOrShadowRootImpl::new(
                document.window(),
                has_browsing_context,
            ),
            has_browsing_context,
            host: Dom::from_ref(host),
            stylesheet_list: MutNullableDom::new(None),
            window: Dom::from_ref(document.window()),
        }
    }

    pub fn new(host: &Element, document: &Document) -> DomRoot<ShadowRoot> {
        reflect_dom_object(
            Box::new(ShadowRoot::new_inherited(host, document)),
            document.window(),
            ShadowRootBinding::Wrap,
        )
    }

    pub fn get_focused_element(&self) -> Option<DomRoot<Element>> {
        //XXX get retargeted focused element
        None
    }

    pub fn stylesheet_count(&self) -> usize {
        //XXX handle shadowroot stylesheets
        0
    }

    pub fn stylesheet_at(&self, _index: usize) -> Option<DomRoot<CSSStyleSheet>> {
        //XXX handle shadowroot stylesheets
        None
    }
}

impl ShadowRootMethods for ShadowRoot {
    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(&self) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root
            .get_active_element(self.get_focused_element(), None, None)
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
    fn ElementFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Option<DomRoot<Element>> {
        // XXX return the result of running the retargeting algorithm with context object
        // and the original result as input
        self.document_or_shadow_root.element_from_point(x, y, None)
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    fn ElementsFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Vec<DomRoot<Element>> {
        // XXX return the result of running the retargeting algorithm with context object
        // and the original result as input
        self.document_or_shadow_root.elements_from_point(x, y, None)
    }

    /// https://dom.spec.whatwg.org/#dom-shadowroot-mode
    fn Mode(&self) -> ShadowRootMode {
        ShadowRootMode::Closed
    }

    /// https://dom.spec.whatwg.org/#dom-shadowroot-host
    fn Host(&self) -> DomRoot<Element> {
        DomRoot::from_ref(&self.host)
    }

    // https://drafts.csswg.org/cssom/#dom-document-stylesheets
    fn StyleSheets(&self) -> DomRoot<StyleSheetList> {
        self.stylesheet_list.or_init(|| {
            StyleSheetList::new(
                &self.window,
                DocumentOrShadowRoot::ShadowRoot(Dom::from_ref(self)),
            )
        })
    }
}
