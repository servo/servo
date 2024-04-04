/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_layout_interface::wrapper_traits::LayoutNode;
use style::context::{SharedStyleContext, StyleContext};
use style::data::ElementData;
use style::dom::{NodeInfo, TElement, TNode};
use style::traversal::{recalc_style_at, DomTraversal, PerLevelTraversalData};

use crate::context::LayoutContext;
use crate::dom::DOMLayoutData;

pub struct RecalcStyle<'a> {
    context: LayoutContext<'a>,
}

impl<'a> RecalcStyle<'a> {
    pub fn new(context: LayoutContext<'a>) -> Self {
        RecalcStyle { context }
    }

    pub fn context(&self) -> &LayoutContext<'a> {
        &self.context
    }

    pub fn destroy(self) -> LayoutContext<'a> {
        self.context
    }
}

#[allow(unsafe_code)]
impl<'a, 'dom, E> DomTraversal<E> for RecalcStyle<'a>
where
    E: TElement,
    E::ConcreteNode: 'dom + LayoutNode<'dom>,
{
    fn process_preorder<F>(
        &self,
        traversal_data: &PerLevelTraversalData,
        context: &mut StyleContext<E>,
        node: E::ConcreteNode,
        note_child: F,
    ) where
        F: FnMut(E::ConcreteNode),
    {
        unsafe {
            node.initialize_style_and_layout_data::<DOMLayoutData>();
            if !node.is_text_node() {
                let el = node.as_element().unwrap();
                let mut data = el.mutate_data().unwrap();
                recalc_style_at(self, traversal_data, context, el, &mut data, note_child);
                el.unset_dirty_descendants();
            }
        }
    }

    #[inline]
    fn needs_postorder_traversal() -> bool {
        false
    }

    fn process_postorder(&self, _style_context: &mut StyleContext<E>, _node: E::ConcreteNode) {
        panic!("this should never be called")
    }

    fn text_node_needs_traversal(node: E::ConcreteNode, parent_data: &ElementData) -> bool {
        node.layout_data().is_none() || !parent_data.damage.is_empty()
    }

    fn shared_context(&self) -> &SharedStyleContext {
        &self.context.style_context
    }
}
