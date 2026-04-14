/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(unsafe_code)]
#![deny(missing_docs)]

use std::borrow::Cow;
use std::fmt;

use layout_api::{
    GenericLayoutData, HTMLCanvasData, HTMLMediaData, LayoutDataTrait, LayoutElement, LayoutNode,
    LayoutNodeType, PseudoElementChain, SVGElementData, SharedSelection, TrustedNodeAddress,
};
use net_traits::image_cache::Image;
use pixels::ImageMetadata;
use servo_arc::Arc;
use servo_base::id::{BrowsingContextId, PipelineId};
use servo_url::ServoUrl;
use style;
use style::context::SharedStyleContext;
use style::dom::{LayoutIterator, NodeInfo};
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;

use super::ServoLayoutElement;
use crate::dom::bindings::root::LayoutDom;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeFlags, NodeTypeIdWrapper};
use crate::layout_dom::{
    ServoDangerousStyleElement, ServoDangerousStyleNode, ServoLayoutNodeChildrenIterator,
};

impl fmt::Debug for LayoutDom<'_, Node> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(element) = self.downcast::<Element>() {
            element.fmt(f)
        } else if self.is_text_node_for_layout() {
            write!(f, "<text node> ({:#x})", self.opaque().0)
        } else {
            write!(f, "<non-text node> ({:#x})", self.opaque().0)
        }
    }
}

/// A wrapper around a `LayoutDom<Node>` which provides a safe interface that
/// can be used during layout. This implements the `LayoutNode` trait.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ServoLayoutNode<'dom> {
    /// The wrapped private DOM node.
    pub(super) node: LayoutDom<'dom, Node>,
    /// The possibly nested [`PseudoElementChain`] for this node.
    pub(super) pseudo_element_chain: PseudoElementChain,
}

/// Those are supposed to be sound, but they aren't because the entire system
/// between script and layout so far has been designed to work around their
/// absence. Switching the entire thing to the inert crate infra will help.
unsafe impl Send for ServoLayoutNode<'_> {}
unsafe impl Sync for ServoLayoutNode<'_> {}

impl<'dom> ServoLayoutNode<'dom> {
    /// Create a new [`ServoLayoutNode`] for this given [`TrustedNodeAddress`].
    ///
    /// # Safety
    ///
    /// The address pointed to by `address` should point to a valid node in memory.
    pub unsafe fn new(address: &TrustedNodeAddress) -> Self {
        unsafe { LayoutDom::from_trusted_node_address(*address) }.into()
    }

    /// Get the first child of this node.
    ///
    /// # Safety
    ///
    /// This node should never be exposed directly to the layout interface, as that may allow
    /// mutating a node that is being laid out in another thread. Thus, this should *never* be
    /// made public or exposed in the `LayoutNode` trait.
    pub(super) unsafe fn dangerous_first_child(&self) -> Option<Self> {
        self.node.first_child_ref().map(Into::into)
    }

    /// Get the next sibling of this node.
    ///
    /// # Safety
    ///
    /// This node should never be exposed directly to the layout interface, as that may allow
    /// mutating a node that is being laid out in another thread. Thus, this should *never* be
    /// made public or exposed in the `LayoutNode` trait.
    pub(super) unsafe fn dangerous_next_sibling(&self) -> Option<Self> {
        self.node.next_sibling_ref().map(Into::into)
    }

    /// Get the previous sibling of this node.
    ///
    /// # Safety
    ///
    /// This node should never be exposed directly to the layout interface, as that may allow
    /// mutating a node that is being laid out in another thread. Thus, this should *never* be
    /// made public or exposed in the `LayoutNode` trait.
    pub(super) unsafe fn dangerous_previous_sibling(&self) -> Option<Self> {
        self.node.prev_sibling_ref().map(Into::into)
    }
}

impl<'dom> From<LayoutDom<'dom, Node>> for ServoLayoutNode<'dom> {
    fn from(node: LayoutDom<'dom, Node>) -> Self {
        Self {
            node,
            pseudo_element_chain: Default::default(),
        }
    }
}

impl<'dom> LayoutNode<'dom> for ServoLayoutNode<'dom> {
    type ConcreteDangerousStyleNode = ServoDangerousStyleNode<'dom>;
    type ConcreteDangerousStyleElement = ServoDangerousStyleElement<'dom>;
    type ConcreteLayoutElement = ServoLayoutElement<'dom>;
    type ChildIterator = ServoLayoutNodeChildrenIterator<'dom>;

    fn with_pseudo(&self, pseudo_element_type: PseudoElement) -> Option<Self> {
        Some(
            self.as_element()?
                .with_pseudo(pseudo_element_type)?
                .as_node(),
        )
    }

    unsafe fn dangerous_style_node(self) -> Self::ConcreteDangerousStyleNode {
        self.node.into()
    }

    unsafe fn dangerous_dom_parent(self) -> Option<Self> {
        self.node.parent_node_ref().map(Into::into)
    }

    unsafe fn dangerous_flat_tree_parent(self) -> Option<Self> {
        self.node
            .traversal_parent()
            .map(|parent_element| parent_element.upcast().into())
    }

    fn is_connected(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_CONNECTED) }
    }

    fn layout_data(&self) -> Option<&'dom GenericLayoutData> {
        self.node.layout_data()
    }

    fn opaque(&self) -> style::dom::OpaqueNode {
        self.node.opaque()
    }

    fn pseudo_element_chain(&self) -> PseudoElementChain {
        self.pseudo_element_chain
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        if self.pseudo_element_chain.is_empty() {
            Some(NodeTypeIdWrapper(self.node.type_id_for_layout()).into())
        } else {
            None
        }
    }

    fn style(&self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        if let Some(element) = self.as_element() {
            element.style(context)
        } else {
            // Text nodes are not styled during traversal,instead we simply
            // return parent style here and do cascading during layout.
            debug_assert!(self.is_text_node());
            self.parent_style(context)
        }
    }

    fn parent_style(&self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        if let Some(chain) = self.pseudo_element_chain.without_innermost() {
            let mut parent = *self;
            parent.pseudo_element_chain = chain;
            return parent.style(context);
        }
        unsafe { self.dangerous_flat_tree_parent() }
            .unwrap()
            .style(context)
    }

    fn selected_style(&self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        let Some(element) = self.as_element() else {
            // TODO(stshine): What should the selected style be for text?
            debug_assert!(self.is_text_node());
            return self.parent_style(context);
        };

        let style_data = &element.element_data().styles;
        let get_selected_style = || {
            // This is a workaround for handling the `::selection` pseudos where it would not
            // propagate to the children and Shadow DOM elements. For this case, UA widget
            // inner elements should follow the originating element in terms of selection.
            if self.node.is_in_ua_widget() {
                return Some(
                    Self::from(
                        self.node
                            .containing_shadow_root_for_layout()?
                            .get_host_for_layout()
                            .upcast(),
                    )
                    .selected_style(context),
                );
            }
            style_data.pseudos.get(&PseudoElement::Selection).cloned()
        };

        get_selected_style().unwrap_or_else(|| style_data.primary().clone())
    }

    fn initialize_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self) {
        if self.node.layout_data().is_none() {
            unsafe {
                self.node
                    .initialize_layout_data(Box::<RequestedLayoutDataType>::default());
            }
        }
    }

    fn flat_tree_children(&self) -> LayoutIterator<ServoLayoutNodeChildrenIterator<'dom>> {
        LayoutIterator(ServoLayoutNodeChildrenIterator::new_for_flat_tree(*self))
    }

    fn dom_children(&self) -> LayoutIterator<ServoLayoutNodeChildrenIterator<'dom>> {
        LayoutIterator(ServoLayoutNodeChildrenIterator::new_for_dom_tree(*self))
    }

    fn as_element(&self) -> Option<ServoLayoutElement<'dom>> {
        self.node.downcast().map(|element| ServoLayoutElement {
            element,
            pseudo_element_chain: self.pseudo_element_chain,
        })
    }

    fn as_html_element(&self) -> Option<ServoLayoutElement<'dom>> {
        self.as_element()
            .filter(|element| element.is_html_element())
    }

    fn text_content(self) -> Cow<'dom, str> {
        self.node.text_content()
    }

    fn selection(&self) -> Option<SharedSelection> {
        self.node.selection()
    }

    fn image_url(&self) -> Option<ServoUrl> {
        self.node.image_url()
    }

    fn image_density(&self) -> Option<f64> {
        self.node.image_density()
    }

    fn showing_broken_image_icon(&self) -> bool {
        self.node.showing_broken_image_icon()
    }

    fn image_data(&self) -> Option<(Option<Image>, Option<ImageMetadata>)> {
        self.node.image_data()
    }

    fn canvas_data(&self) -> Option<HTMLCanvasData> {
        self.node.canvas_data()
    }

    fn media_data(&self) -> Option<HTMLMediaData> {
        self.node.media_data()
    }

    fn svg_data(&self) -> Option<SVGElementData<'dom>> {
        self.node.svg_data()
    }

    fn iframe_browsing_context_id(&self) -> Option<BrowsingContextId> {
        self.node.iframe_browsing_context_id()
    }

    fn iframe_pipeline_id(&self) -> Option<PipelineId> {
        self.node.iframe_pipeline_id()
    }

    fn table_span(&self) -> Option<u32> {
        self.node
            .downcast::<Element>()
            .and_then(|element| element.get_span())
    }

    fn table_colspan(&self) -> Option<u32> {
        self.node
            .downcast::<Element>()
            .and_then(|element| element.get_colspan())
    }

    fn table_rowspan(&self) -> Option<u32> {
        self.node
            .downcast::<Element>()
            .and_then(|element| element.get_rowspan())
    }

    fn set_uses_content_attribute_with_attr(&self, uses_content_attribute_with_attr: bool) {
        unsafe {
            self.node.set_flag(
                NodeFlags::USES_ATTR_IN_CONTENT_ATTRIBUTE,
                uses_content_attribute_with_attr,
            )
        }
    }

    fn is_single_line_text_input(&self) -> bool {
        self.pseudo_element_chain.is_empty() && self.node.is_text_container_of_single_line_input()
    }

    fn is_root_of_user_agent_widget(&self) -> bool {
        self.node.is_root_of_user_agent_widget()
    }
}

impl NodeInfo for ServoLayoutNode<'_> {
    fn is_element(&self) -> bool {
        self.node.is_element_for_layout()
    }

    fn is_text_node(&self) -> bool {
        self.node.is_text_node_for_layout()
    }
}
