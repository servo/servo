/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use euclid::default::Point2D;
use script_layout_interface::{NodesFromPointQueryType, QueryMsg};
use script_traits::UntrustedNodeAddress;
use servo_arc::Arc;
use servo_atoms::Atom;
use style::invalidation::media_queries::{MediaListKey, ToMediaListKey};
use style::media_queries::MediaList;
use style::shared_lock::{SharedRwLock as StyleSharedRwLock, SharedRwLockReadGuard};
use style::stylesheets::scope_rule::ImplicitScopeRoot;
use style::stylesheets::{Stylesheet, StylesheetContents};

use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{self, Node, VecPreOrderInsertionHelper};
use crate::dom::window::Window;
use crate::stylesheet_set::StylesheetSetRef;

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct StyleSheetInDocument {
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    pub sheet: Arc<Stylesheet>,
    pub owner: Dom<Element>,
}

impl fmt::Debug for StyleSheetInDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.sheet.fmt(formatter)
    }
}

impl PartialEq for StyleSheetInDocument {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.sheet, &other.sheet)
    }
}

impl ToMediaListKey for StyleSheetInDocument {
    fn to_media_list_key(&self) -> MediaListKey {
        self.sheet.contents.to_media_list_key()
    }
}

impl ::style::stylesheets::StylesheetInDocument for StyleSheetInDocument {
    fn enabled(&self) -> bool {
        self.sheet.enabled()
    }

    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.sheet.media(guard)
    }

    fn contents(&self) -> &StylesheetContents {
        self.sheet.contents()
    }

    fn implicit_scope_root(&self) -> Option<ImplicitScopeRoot> {
        None
    }
}

// https://w3c.github.io/webcomponents/spec/shadow/#extensions-to-the-documentorshadowroot-mixin
#[crown::unrooted_must_root_lint::must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub struct DocumentOrShadowRoot {
    window: Dom<Window>,
}

impl DocumentOrShadowRoot {
    pub fn new(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
        }
    }

    pub fn nodes_from_point(
        &self,
        client_point: &Point2D<f32>,
        query_type: NodesFromPointQueryType,
    ) -> Vec<UntrustedNodeAddress> {
        if !self.window.layout_reflow(QueryMsg::NodesFromPointQuery) {
            return vec![];
        };

        self.window
            .layout()
            .query_nodes_from_point(*client_point, query_type)
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

        if !has_browsing_context {
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
                let node = unsafe { node::from_untrusted_node_address(*address) };
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

        if !has_browsing_context {
            return vec![];
        }

        // Step 2
        if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
            return vec![];
        }

        // Step 1 and Step 3
        let nodes = self.nodes_from_point(point, NodesFromPointQueryType::All);
        let mut elements: Vec<DomRoot<Element>> = nodes
            .iter()
            .flat_map(|&untrusted_node_address| {
                let node = unsafe { node::from_untrusted_node_address(untrusted_node_address) };
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

    /// Remove a stylesheet owned by `owner` from the list of document sheets.
    #[allow(crown::unrooted_must_root)] // Owner needs to be rooted already necessarily.
    pub fn remove_stylesheet(
        owner: &Element,
        s: &Arc<Stylesheet>,
        mut stylesheets: StylesheetSetRef<StyleSheetInDocument>,
    ) {
        let guard = s.shared_lock.read();

        // FIXME(emilio): Would be nice to remove the clone, etc.
        stylesheets.remove_stylesheet(
            None,
            StyleSheetInDocument {
                sheet: s.clone(),
                owner: Dom::from_ref(owner),
            },
            &guard,
        );
    }

    /// Add a stylesheet owned by `owner` to the list of document sheets, in the
    /// correct tree position.
    #[allow(crown::unrooted_must_root)] // Owner needs to be rooted already necessarily.
    pub fn add_stylesheet(
        owner: &Element,
        mut stylesheets: StylesheetSetRef<StyleSheetInDocument>,
        sheet: Arc<Stylesheet>,
        insertion_point: Option<StyleSheetInDocument>,
        style_shared_lock: &StyleSharedRwLock,
    ) {
        debug_assert!(owner.as_stylesheet_owner().is_some(), "Wat");

        let sheet = StyleSheetInDocument {
            sheet,
            owner: Dom::from_ref(owner),
        };

        let guard = style_shared_lock.read();

        match insertion_point {
            Some(ip) => {
                stylesheets.insert_stylesheet_before(None, sheet, ip, &guard);
            },
            None => {
                stylesheets.append_stylesheet(None, sheet, &guard);
            },
        }
    }

    /// Remove any existing association between the provided id/name and any elements in this document.
    pub fn unregister_named_element(
        &self,
        id_map: &DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>>>,
        to_unregister: &Element,
        id: &Atom,
    ) {
        debug!(
            "Removing named element {:p}: {:p} id={}",
            self, to_unregister, id
        );
        let mut id_map = id_map.borrow_mut();
        let is_empty = match id_map.get_mut(id) {
            None => false,
            Some(elements) => {
                let position = elements
                    .iter()
                    .position(|element| &**element == to_unregister)
                    .expect("This element should be in registered.");
                elements.remove(position);
                elements.is_empty()
            },
        };
        if is_empty {
            id_map.remove(id);
        }
    }

    /// Associate an element present in this document with the provided id/name.
    pub fn register_named_element(
        &self,
        id_map: &DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>>>,
        element: &Element,
        id: &Atom,
        root: DomRoot<Node>,
    ) {
        debug!("Adding named element {:p}: {:p} id={}", self, element, id);
        assert!(element.upcast::<Node>().is_connected());
        assert!(!id.is_empty());
        let mut id_map = id_map.borrow_mut();
        let elements = id_map.entry(id.clone()).or_default();
        elements.insert_pre_order(element, &root);
    }
}
