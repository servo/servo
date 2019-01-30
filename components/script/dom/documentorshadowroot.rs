/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::window::Window;
use euclid::Point2D;
use js::jsapi::JS_GetRuntime;
use script_layout_interface::message::{NodesFromPointQueryType, QueryMsg};
use script_traits::UntrustedNodeAddress;

macro_rules! proxy_call(
    ($fn_name:ident, $return_type:ty) => (
        pub fn $fn_name(&self) -> $return_type {
            match self {
                DocumentOrShadowRoot::Document(doc) => doc.$fn_name(),
                DocumentOrShadowRoot::ShadowRoot(root) => root.$fn_name(),
            }
        }
    );

    ($fn_name:ident, $arg1:ident, $arg1_type:ty, $return_type:ty) => (
        pub fn $fn_name(&self, $arg1: $arg1_type) -> $return_type {
            match self {
                DocumentOrShadowRoot::Document(doc) => doc.$fn_name($arg1),
                DocumentOrShadowRoot::ShadowRoot(root) => root.$fn_name($arg1),
            }
        }
    );
);

#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub enum DocumentOrShadowRoot {
    Document(Dom<Document>),
    ShadowRoot(Dom<ShadowRoot>),
}

impl DocumentOrShadowRoot {
    proxy_call!(stylesheet_count, usize);
    proxy_call!(stylesheet_at, index, usize, Option<DomRoot<CSSStyleSheet>>);
}

// https://w3c.github.io/webcomponents/spec/shadow/#extensions-to-the-documentorshadowroot-mixin
#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub struct DocumentOrShadowRootImpl {
    window: Dom<Window>,
}

impl DocumentOrShadowRootImpl {
    pub fn new(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
        }
    }

    pub fn nodes_from_point(
        &self,
        client_point: &Point2D<f32>,
        reflow_goal: NodesFromPointQueryType,
    ) -> Vec<UntrustedNodeAddress> {
        if !self
            .window
            .layout_reflow(QueryMsg::NodesFromPointQuery(*client_point, reflow_goal))
        {
            return vec![];
        };

        self.window.layout().nodes_from_point_response()
    }

    #[allow(unsafe_code)]
    // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
    pub fn element_from_point(
        &self,
        x: Finite<f64>,
        y: Finite<f64>,
        document_element: Option<DomRoot<Element>>,
        has_browsing_context: bool,
    ) -> Option<DomRoot<Element>> {
        let x = *x as f32;
        let y = *y as f32;
        let point = &Point2D::new(x, y);
        let viewport = self.window.window_size().initial_viewport;

        if has_browsing_context {
            return None;
        }

        if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
            return None;
        }

        match self
            .nodes_from_point(point, NodesFromPointQueryType::Topmost)
            .first()
        {
            Some(address) => {
                let js_runtime = unsafe { JS_GetRuntime(self.window.get_cx()) };
                let node = unsafe { node::from_untrusted_node_address(js_runtime, *address) };
                let parent_node = node.GetParentNode().unwrap();
                let element_ref = node
                    .downcast::<Element>()
                    .unwrap_or_else(|| parent_node.downcast::<Element>().unwrap());

                Some(DomRoot::from_ref(element_ref))
            },
            None => document_element,
        }
    }

    #[allow(unsafe_code)]
    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    pub fn elements_from_point(
        &self,
        x: Finite<f64>,
        y: Finite<f64>,
        document_element: Option<DomRoot<Element>>,
        has_browsing_context: bool,
    ) -> Vec<DomRoot<Element>> {
        let x = *x as f32;
        let y = *y as f32;
        let point = &Point2D::new(x, y);
        let viewport = self.window.window_size().initial_viewport;

        if has_browsing_context {
            return vec![];
        }

        // Step 2
        if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
            return vec![];
        }

        let js_runtime = unsafe { JS_GetRuntime(self.window.get_cx()) };

        // Step 1 and Step 3
        let nodes = self.nodes_from_point(point, NodesFromPointQueryType::All);
        let mut elements: Vec<DomRoot<Element>> = nodes
            .iter()
            .flat_map(|&untrusted_node_address| {
                let node = unsafe {
                    node::from_untrusted_node_address(js_runtime, untrusted_node_address)
                };
                DomRoot::downcast::<Element>(node)
            })
            .collect();

        // Step 4
        if let Some(root_element) = document_element {
            if elements.last() != Some(&root_element) {
                elements.push(root_element);
            }
        }

        // Step 5
        elements
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    pub fn get_active_element(
        &self,
        focused_element: Option<DomRoot<Element>>,
        body: Option<DomRoot<HTMLElement>>,
        document_element: Option<DomRoot<Element>>,
    ) -> Option<DomRoot<Element>> {
        // TODO: Step 2.

        match focused_element {
            Some(element) => Some(element), // Step 3. and 4.
            None => match body {
                // Step 5.
                Some(body) => Some(DomRoot::upcast(body)),
                None => document_element,
            },
        }
    }
}
