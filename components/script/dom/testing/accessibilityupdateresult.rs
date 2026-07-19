/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::ServoTestUtilsBinding::AccessibilityUpdateResultMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct AccessibilityUpdateResult {
    reflector_: Reflector,
    nodes_updated_from_dom: u32,
    nodes_updated_from_tree: u32,
    nodes_in_tree_update: u32,
}

impl AccessibilityUpdateResult {
    pub(crate) fn new_inherited(
        nodes_updated_from_dom: u32,
        nodes_updated_from_tree: u32,
        nodes_in_tree_update: u32,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            nodes_updated_from_dom,
            nodes_updated_from_tree,
            nodes_in_tree_update,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        nodes_updated_from_dom: u32,
        nodes_updated_from_tree: u32,
        nodes_in_tree_update: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(Self::new_inherited(
                nodes_updated_from_dom,
                nodes_updated_from_tree,
                nodes_in_tree_update,
            )),
            global,
            cx,
        )
    }
}

impl AccessibilityUpdateResultMethods<crate::DomTypeHolder> for AccessibilityUpdateResult {
    fn NodesUpdatedFromDom(&self) -> u32 {
        self.nodes_updated_from_dom
    }

    fn NodesUpdatedFromTree(&self) -> u32 {
        self.nodes_updated_from_tree
    }

    fn NodesInTreeUpdate(&self) -> u32 {
        self.nodes_in_tree_update
    }
}
