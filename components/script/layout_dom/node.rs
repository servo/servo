/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::fmt;
use std::sync::Arc as StdArc;

use base::id::{BrowsingContextId, PipelineId};
use fonts_traits::ByteIndex;
use html5ever::{local_name, ns};
use pixels::{Image, ImageMetadata};
use range::Range;
use script_layout_interface::wrapper_traits::{LayoutDataTrait, LayoutNode, ThreadSafeLayoutNode};
use script_layout_interface::{
    GenericLayoutData, HTMLCanvasData, HTMLMediaData, LayoutNodeType, SVGSVGData, StyleData,
    TrustedNodeAddress,
};
use servo_arc::Arc;
use servo_url::ServoUrl;
use style;
use style::dom::{NodeInfo, TElement, TNode, TShadowRoot};
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;

use super::{
    ServoLayoutDocument, ServoLayoutElement, ServoShadowRoot, ServoThreadSafeLayoutElement,
};
use crate::dom::bindings::inheritance::{CharacterDataTypeId, NodeTypeId, TextTypeId};
use crate::dom::bindings::root::LayoutDom;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::node::{LayoutNodeHelpers, Node, NodeFlags, NodeTypeIdWrapper};

/// A wrapper around a `LayoutDom<Node>` which provides a safe interface that
/// can be used during layout. This implements the `LayoutNode` trait as well as
/// several style and selectors traits for use during layout. This version
/// should only be used on a single thread. If you need to use nodes across
/// threads use ServoThreadSafeLayoutNode.
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct ServoLayoutNode<'dom> {
    /// The wrapped private DOM node.
    pub(super) node: LayoutDom<'dom, Node>,
}

/// Those are supposed to be sound, but they aren't because the entire system
/// between script and layout so far has been designed to work around their
/// absence. Switching the entire thing to the inert crate infra will help.
///
/// FIXME(mrobinson): These are required because Layout 2020 sends non-threadsafe
/// nodes to different threads. This should be adressed in a comprehensive way.
unsafe impl Send for ServoLayoutNode<'_> {}
unsafe impl Sync for ServoLayoutNode<'_> {}

impl fmt::Debug for ServoLayoutNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(el) = self.as_element() {
            el.fmt(f)
        } else if self.is_text_node() {
            write!(f, "<text node> ({:#x})", self.opaque().0)
        } else {
            write!(f, "<non-text node> ({:#x})", self.opaque().0)
        }
    }
}

impl<'dom> ServoLayoutNode<'dom> {
    pub(super) fn from_layout_js(n: LayoutDom<'dom, Node>) -> Self {
        ServoLayoutNode { node: n }
    }

    /// Create a new [`ServoLayoutNode`] for this given [`TrustedNodeAddress`].
    ///
    /// # Safety
    ///
    /// The address pointed to by `address` should point to a valid node in memory.
    pub unsafe fn new(address: &TrustedNodeAddress) -> Self {
        ServoLayoutNode::from_layout_js(LayoutDom::from_trusted_node_address(*address))
    }

    pub(super) fn script_type_id(&self) -> NodeTypeId {
        self.node.type_id_for_layout()
    }

    /// Returns the interior of this node as a `LayoutDom`.
    pub(crate) fn get_jsmanaged(self) -> LayoutDom<'dom, Node> {
        self.node
    }

    pub(crate) fn assigned_slot(self) -> Option<ServoLayoutElement<'dom>> {
        self.node
            .assigned_slot_for_layout()
            .as_ref()
            .map(LayoutDom::upcast)
            .map(ServoLayoutElement::from_layout_js)
    }

    pub fn is_text_input(&self) -> bool {
        self.node.is_text_input()
    }
}

impl style::dom::NodeInfo for ServoLayoutNode<'_> {
    fn is_element(&self) -> bool {
        self.node.is_element_for_layout()
    }

    fn is_text_node(&self) -> bool {
        self.script_type_id() ==
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text))
    }
}

impl<'dom> style::dom::TNode for ServoLayoutNode<'dom> {
    type ConcreteDocument = ServoLayoutDocument<'dom>;
    type ConcreteElement = ServoLayoutElement<'dom>;
    type ConcreteShadowRoot = ServoShadowRoot<'dom>;

    fn parent_node(&self) -> Option<Self> {
        self.node
            .composed_parent_node_ref()
            .map(Self::from_layout_js)
    }

    fn first_child(&self) -> Option<Self> {
        self.node.first_child_ref().map(Self::from_layout_js)
    }

    fn last_child(&self) -> Option<Self> {
        self.node.last_child_ref().map(Self::from_layout_js)
    }

    fn prev_sibling(&self) -> Option<Self> {
        self.node.prev_sibling_ref().map(Self::from_layout_js)
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling_ref().map(Self::from_layout_js)
    }

    fn owner_doc(&self) -> Self::ConcreteDocument {
        ServoLayoutDocument::from_layout_js(self.node.owner_doc_for_layout())
    }

    fn traversal_parent(&self) -> Option<ServoLayoutElement<'dom>> {
        if let Some(assigned_slot) = self.assigned_slot() {
            return Some(assigned_slot);
        }
        let parent = self.parent_node()?;
        if let Some(shadow) = parent.as_shadow_root() {
            return Some(shadow.host());
        };
        parent.as_element()
    }

    fn opaque(&self) -> style::dom::OpaqueNode {
        self.get_jsmanaged().opaque()
    }

    fn debug_id(self) -> usize {
        self.opaque().0
    }

    fn as_element(&self) -> Option<ServoLayoutElement<'dom>> {
        self.node.downcast().map(ServoLayoutElement::from_layout_js)
    }

    fn as_document(&self) -> Option<ServoLayoutDocument<'dom>> {
        self.node
            .downcast()
            .map(ServoLayoutDocument::from_layout_js)
    }

    fn as_shadow_root(&self) -> Option<ServoShadowRoot<'dom>> {
        self.node.downcast().map(ServoShadowRoot::from_layout_js)
    }

    fn is_in_document(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_IN_A_DOCUMENT_TREE) }
    }
}

impl<'dom> LayoutNode<'dom> for ServoLayoutNode<'dom> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'dom>;

    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode {
        ServoThreadSafeLayoutNode::new(*self)
    }

    fn type_id(&self) -> LayoutNodeType {
        NodeTypeIdWrapper(self.script_type_id()).into()
    }

    unsafe fn initialize_style_and_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self) {
        let inner = self.get_jsmanaged();
        if inner.style_data().is_none() {
            inner.initialize_style_data();
        }
        if inner.layout_data().is_none() {
            inner.initialize_layout_data(Box::<RequestedLayoutDataType>::default());
        }
    }

    fn initialize_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self) {
        let inner = self.get_jsmanaged();
        if inner.layout_data().is_none() {
            unsafe {
                inner.initialize_layout_data(Box::<RequestedLayoutDataType>::default());
            }
        }
    }

    fn is_connected(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_CONNECTED) }
    }

    fn style_data(&self) -> Option<&'dom StyleData> {
        self.get_jsmanaged().style_data()
    }

    fn layout_data(&self) -> Option<&'dom GenericLayoutData> {
        self.get_jsmanaged().layout_data()
    }
}

/// A wrapper around a `ServoLayoutNode` that can be used safely on different threads.
/// It's very important that this never mutate anything except this wrapped node and
/// never access any other node apart from its parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServoThreadSafeLayoutNode<'dom> {
    /// The wrapped `ServoLayoutNode`.
    pub(super) node: ServoLayoutNode<'dom>,

    /// The pseudo-element type for this node, or `None` if it is the non-pseudo
    /// version of the element.
    pub(super) pseudo: Option<PseudoElement>,
}

impl<'dom> ServoThreadSafeLayoutNode<'dom> {
    /// Creates a new `ServoThreadSafeLayoutNode` from the given `ServoLayoutNode`.
    pub fn new(node: ServoLayoutNode<'dom>) -> Self {
        ServoThreadSafeLayoutNode { node, pseudo: None }
    }

    /// Returns the interior of this node as a `LayoutDom`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged(&self) -> LayoutDom<'dom, Node> {
        self.node.get_jsmanaged()
    }

    /// Get the first child of this node. Important: this is not safe for
    /// layout to call, so it should *never* be made public.
    unsafe fn dangerous_first_child(&self) -> Option<Self> {
        self.get_jsmanaged()
            .first_child_ref()
            .map(ServoLayoutNode::from_layout_js)
            .map(Self::new)
    }

    /// Get the next sibling of this node. Important: this is not safe for
    /// layout to call, so it should *never* be made public.
    unsafe fn dangerous_next_sibling(&self) -> Option<Self> {
        self.get_jsmanaged()
            .next_sibling_ref()
            .map(ServoLayoutNode::from_layout_js)
            .map(Self::new)
    }
}

impl style::dom::NodeInfo for ServoThreadSafeLayoutNode<'_> {
    fn is_element(&self) -> bool {
        self.node.is_element()
    }

    fn is_text_node(&self) -> bool {
        self.node.is_text_node()
    }
}

impl<'dom> ThreadSafeLayoutNode<'dom> for ServoThreadSafeLayoutNode<'dom> {
    type ConcreteNode = ServoLayoutNode<'dom>;
    type ConcreteThreadSafeLayoutElement = ServoThreadSafeLayoutElement<'dom>;
    type ConcreteElement = ServoLayoutElement<'dom>;
    type ChildrenIterator = ServoThreadSafeLayoutNodeChildrenIterator<'dom>;

    fn opaque(&self) -> style::dom::OpaqueNode {
        unsafe { self.get_jsmanaged().opaque() }
    }

    fn pseudo_element(&self) -> Option<PseudoElement> {
        self.pseudo
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        if self.pseudo.is_none() {
            Some(self.node.type_id())
        } else {
            None
        }
    }

    fn parent_style(&self) -> Arc<ComputedValues> {
        let parent = self.node.parent_node().unwrap().as_element().unwrap();
        let parent_data = parent.borrow_data().unwrap();
        parent_data.styles.primary().clone()
    }

    fn debug_id(self) -> usize {
        self.node.debug_id()
    }

    fn children(&self) -> style::dom::LayoutIterator<Self::ChildrenIterator> {
        if let Some(shadow) = self.node.as_element().and_then(|e| e.shadow_root()) {
            return style::dom::LayoutIterator(ServoThreadSafeLayoutNodeChildrenIterator::new(
                shadow.as_node().to_threadsafe(),
            ));
        }
        style::dom::LayoutIterator(ServoThreadSafeLayoutNodeChildrenIterator::new(*self))
    }

    fn as_element(&self) -> Option<ServoThreadSafeLayoutElement<'dom>> {
        self.node
            .as_element()
            .map(|el| ServoThreadSafeLayoutElement {
                element: el,
                pseudo: self.pseudo,
            })
    }

    fn as_html_element(&self) -> Option<ServoThreadSafeLayoutElement<'dom>> {
        self.as_element()
            .filter(|element| element.element.is_html_element())
    }

    fn style_data(&self) -> Option<&'dom StyleData> {
        self.node.style_data()
    }

    fn layout_data(&self) -> Option<&'dom GenericLayoutData> {
        self.node.layout_data()
    }

    fn unsafe_get(self) -> Self::ConcreteNode {
        self.node
    }

    fn node_text_content(self) -> Cow<'dom, str> {
        unsafe { self.get_jsmanaged().text_content() }
    }

    fn selection(&self) -> Option<Range<ByteIndex>> {
        let this = unsafe { self.get_jsmanaged() };

        this.selection().map(|range| {
            Range::new(
                ByteIndex(range.start as isize),
                ByteIndex(range.len() as isize),
            )
        })
    }

    fn image_url(&self) -> Option<ServoUrl> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_url()
    }

    fn image_density(&self) -> Option<f64> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_density()
    }

    fn image_data(&self) -> Option<(Option<StdArc<Image>>, Option<ImageMetadata>)> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_data()
    }

    fn canvas_data(&self) -> Option<HTMLCanvasData> {
        let this = unsafe { self.get_jsmanaged() };
        this.canvas_data()
    }

    fn media_data(&self) -> Option<HTMLMediaData> {
        let this = unsafe { self.get_jsmanaged() };
        this.media_data()
    }

    fn svg_data(&self) -> Option<SVGSVGData> {
        let this = unsafe { self.get_jsmanaged() };
        this.svg_data()
    }

    // Can return None if the iframe has no nested browsing context
    fn iframe_browsing_context_id(&self) -> Option<BrowsingContextId> {
        let this = unsafe { self.get_jsmanaged() };
        this.iframe_browsing_context_id()
    }

    // Can return None if the iframe has no nested browsing context
    fn iframe_pipeline_id(&self) -> Option<PipelineId> {
        let this = unsafe { self.get_jsmanaged() };
        this.iframe_pipeline_id()
    }

    fn get_span(&self) -> Option<u32> {
        unsafe {
            self.get_jsmanaged()
                .downcast::<Element>()
                .unwrap()
                .get_span()
        }
    }

    fn get_colspan(&self) -> Option<u32> {
        unsafe {
            self.get_jsmanaged()
                .downcast::<Element>()
                .unwrap()
                .get_colspan()
        }
    }

    fn get_rowspan(&self) -> Option<u32> {
        unsafe {
            self.get_jsmanaged()
                .downcast::<Element>()
                .unwrap()
                .get_rowspan()
        }
    }
}

pub struct ServoThreadSafeLayoutNodeChildrenIterator<'dom> {
    current_node: Option<ServoThreadSafeLayoutNode<'dom>>,
    parent_node: ServoThreadSafeLayoutNode<'dom>,
}

impl<'dom> ServoThreadSafeLayoutNodeChildrenIterator<'dom> {
    pub fn new(parent: ServoThreadSafeLayoutNode<'dom>) -> Self {
        let first_child = match parent.pseudo_element() {
            None => parent
                .with_pseudo(PseudoElement::Before)
                .or_else(|| parent.with_pseudo(PseudoElement::DetailsSummary))
                .or_else(|| unsafe { parent.dangerous_first_child() }),
            Some(PseudoElement::DetailsContent) | Some(PseudoElement::DetailsSummary) => unsafe {
                parent.dangerous_first_child()
            },
            _ => None,
        };
        ServoThreadSafeLayoutNodeChildrenIterator {
            current_node: first_child,
            parent_node: parent,
        }
    }
}

impl<'dom> Iterator for ServoThreadSafeLayoutNodeChildrenIterator<'dom> {
    type Item = ServoThreadSafeLayoutNode<'dom>;
    fn next(&mut self) -> Option<ServoThreadSafeLayoutNode<'dom>> {
        use selectors::Element;
        match self.parent_node.pseudo_element() {
            Some(PseudoElement::Before) | Some(PseudoElement::After) => None,

            Some(PseudoElement::DetailsSummary) => {
                let mut current_node = self.current_node;
                loop {
                    let next_node = if let Some(ref node) = current_node {
                        if let Some(element) = node.as_element() {
                            if element.has_local_name(&local_name!("summary")) &&
                                element.has_namespace(&ns!(html))
                            {
                                self.current_node = None;
                                return Some(*node);
                            }
                        }
                        unsafe { node.dangerous_next_sibling() }
                    } else {
                        self.current_node = None;
                        return None;
                    };
                    current_node = next_node;
                }
            },

            Some(PseudoElement::DetailsContent) => {
                let node = self.current_node;
                let node = node.and_then(|node| {
                    if node.is_element() &&
                        node.as_element()
                            .unwrap()
                            .has_local_name(&local_name!("summary")) &&
                        node.as_element().unwrap().has_namespace(&ns!(html))
                    {
                        unsafe { node.dangerous_next_sibling() }
                    } else {
                        Some(node)
                    }
                });
                self.current_node = node.and_then(|node| unsafe { node.dangerous_next_sibling() });
                node
            },

            None | Some(_) => {
                let node = self.current_node;
                if let Some(ref node) = node {
                    self.current_node = match node.pseudo_element() {
                        Some(PseudoElement::Before) => self
                            .parent_node
                            .with_pseudo(PseudoElement::DetailsSummary)
                            .or_else(|| unsafe { self.parent_node.dangerous_first_child() })
                            .or_else(|| self.parent_node.with_pseudo(PseudoElement::After)),
                        Some(PseudoElement::DetailsSummary) => {
                            self.parent_node.with_pseudo(PseudoElement::DetailsContent)
                        },
                        Some(PseudoElement::DetailsContent) => {
                            self.parent_node.with_pseudo(PseudoElement::After)
                        },
                        Some(PseudoElement::After) => None,
                        None | Some(_) => unsafe { node.dangerous_next_sibling() }
                            .or_else(|| self.parent_node.with_pseudo(PseudoElement::After)),
                    };
                }
                node
            },
        }
    }
}
