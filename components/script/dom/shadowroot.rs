/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootMode;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documentorshadowroot::DocumentOrShadowRoot;
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node;
use crate::dom::stylesheetlist::StyleSheetList;
use crate::dom::window::Window;
use crate::dom::windowproxy::WindowProxy;
use dom_struct::dom_struct;
use euclid::Point2D;
use js::jsapi::JS_GetRuntime;
use script_layout_interface::message::{NodesFromPointQueryType, QueryMsg};
use script_traits::UntrustedNodeAddress;

// https://dom.spec.whatwg.org/#interface-shadowroot
#[dom_struct]
pub struct ShadowRoot {
    document_fragment: DocumentFragment,
    has_browsing_context: bool,
    host: Dom<Element>,
    stylesheet_list: MutNullableDom<StyleSheetList>,
    window: Dom<Window>,
}

impl ShadowRoot {
    #[allow(dead_code)]
    pub fn new_inherited(host: &Element, document: &Document) -> ShadowRoot {
        ShadowRoot {
            document_fragment: DocumentFragment::new_inherited(document),
            has_browsing_context: true,
            host: Dom::from_ref(host),
            stylesheet_list: MutNullableDom::new(None),
            window: Dom::from_ref(document.window()),
        }
    }

    pub fn get_focused_element(&self) -> Option<DomRoot<Element>> {
        //XXX get retargeted focused element
        None
    }

    pub fn GetDocumentElement(&self) -> Option<DomRoot<Element>> {
        None
    }

    pub fn GetBody(&self) -> Option<DomRoot<HTMLElement>> {
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

    impl_document_or_shadow_root_helpers!();
}

impl ShadowRootMethods for ShadowRoot {
    /// https://w3c.github.io/webcomponents/spec/shadow/#extensions-to-the-documentorshadowroot-mixin
    impl_document_or_shadow_root_methods!(ShadowRoot);

    /// https://dom.spec.whatwg.org/#dom-shadowroot-mode
    fn Mode(&self) -> ShadowRootMode {
        ShadowRootMode::Closed
    }

    /// https://dom.spec.whatwg.org/#dom-shadowroot-host
    fn Host(&self) -> DomRoot<Element> {
        DomRoot::from_ref(&self.host)
    }
}
