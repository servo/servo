/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc as StdArc;

use atomic_refcell::AtomicRefCell;
use gfx_traits::ByteIndex;
use html5ever::{local_name, namespace_url, ns};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image::base::{Image, ImageMetadata};
use range::Range;
use script_layout_interface::wrapper_traits::{
    GetStyleAndOpaqueLayoutData, LayoutDataTrait, LayoutNode, PseudoElementType,
    ThreadSafeLayoutNode,
};
use script_layout_interface::{
    HTMLCanvasData, HTMLMediaData, LayoutNodeType, SVGSVGData, StyleAndOpaqueLayoutData, StyleData,
    TrustedNodeAddress,
};
use servo_arc::Arc;
use servo_url::ServoUrl;
use style;
use style::context::SharedStyleContext;
use style::dom::{NodeInfo, TElement, TNode, TShadowRoot};
use style::properties::ComputedValues;
use style::str::is_whitespace;

use super::{
    ServoLayoutDocument, ServoLayoutElement, ServoShadowRoot, ServoThreadSafeLayoutElement,
};
use crate::dom::bindings::inheritance::{CharacterDataTypeId, NodeTypeId, TextTypeId};
use crate::dom::bindings::root::LayoutDom;
use crate::dom::characterdata::LayoutCharacterDataHelpers;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::node::{LayoutNodeHelpers, Node, NodeFlags};
use crate::dom::text::Text;

/// A wrapper around a `LayoutDom<Node>` which provides a safe interface that
/// can be used during layout. This implements the `LayoutNode` trait as well as
/// several style and selectors traits for use during layout. This version
/// should only be used on a single thread. If you need to use nodes across
/// threads use ServoThreadSafeLayoutNode.
pub struct ServoLayoutNode<'dom, LayoutDataType: LayoutDataTrait> {
    /// The wrapped private DOM node.
    pub(super) node: LayoutDom<'dom, Node>,

    /// A PhantomData that is used to track the type of the stored layout data.
    pub(super) phantom: PhantomData<LayoutDataType>,
}

//// Those are supposed to be sound, but they aren't because the entire system
//// between script and layout so far has been designed to work around their
//// absence. Switching the entire thing to the inert crate infra will help.
///
//// FIXME(mrobinson): These are required because Layout 2020 sends non-threadsafe
//// nodes to different threads. This should be adressed in a comprehensive way.
unsafe impl<LayoutDataType: LayoutDataTrait> Send for ServoLayoutNode<'_, LayoutDataType> {}
unsafe impl<LayoutDataType: LayoutDataTrait> Sync for ServoLayoutNode<'_, LayoutDataType> {}

// These impls are required because `derive` has trouble with PhantomData.
// See https://github.com/rust-lang/rust/issues/52079
impl<'dom, LayoutDataType: LayoutDataTrait> Clone for ServoLayoutNode<'dom, LayoutDataType> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'dom, LayoutDataType: LayoutDataTrait> Copy for ServoLayoutNode<'dom, LayoutDataType> {}
impl<'a, LayoutDataType: LayoutDataTrait> PartialEq for ServoLayoutNode<'a, LayoutDataType> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> fmt::Debug for ServoLayoutNode<'dom, LayoutDataType> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(el) = self.as_element() {
            el.fmt(f)
        } else {
            if self.is_text_node() {
                write!(f, "<text node> ({:#x})", self.opaque().0)
            } else {
                write!(f, "<non-text node> ({:#x})", self.opaque().0)
            }
        }
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> ServoLayoutNode<'dom, LayoutDataType> {
    pub(super) fn from_layout_js(n: LayoutDom<'dom, Node>) -> Self {
        ServoLayoutNode {
            node: n,
            phantom: PhantomData,
        }
    }

    pub unsafe fn new(address: &TrustedNodeAddress) -> Self {
        ServoLayoutNode::from_layout_js(LayoutDom::from_trusted_node_address(*address))
    }

    pub(super) fn script_type_id(&self) -> NodeTypeId {
        self.node.type_id_for_layout()
    }

    /// Returns the interior of this node as a `LayoutDom`.
    pub fn get_jsmanaged(self) -> LayoutDom<'dom, Node> {
        self.node
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> style::dom::NodeInfo
    for ServoLayoutNode<'dom, LayoutDataType>
{
    fn is_element(&self) -> bool {
        self.node.is_element_for_layout()
    }

    fn is_text_node(&self) -> bool {
        self.script_type_id() ==
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text))
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> style::dom::TNode
    for ServoLayoutNode<'dom, LayoutDataType>
{
    type ConcreteDocument = ServoLayoutDocument<'dom, LayoutDataType>;
    type ConcreteElement = ServoLayoutElement<'dom, LayoutDataType>;
    type ConcreteShadowRoot = ServoShadowRoot<'dom, LayoutDataType>;

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

    fn traversal_parent(&self) -> Option<ServoLayoutElement<'dom, LayoutDataType>> {
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

    fn as_element(&self) -> Option<ServoLayoutElement<'dom, LayoutDataType>> {
        self.node.downcast().map(ServoLayoutElement::from_layout_js)
    }

    fn as_document(&self) -> Option<ServoLayoutDocument<'dom, LayoutDataType>> {
        self.node
            .downcast()
            .map(ServoLayoutDocument::from_layout_js)
    }

    fn as_shadow_root(&self) -> Option<ServoShadowRoot<'dom, LayoutDataType>> {
        self.node.downcast().map(ServoShadowRoot::from_layout_js)
    }

    fn is_in_document(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_IN_DOC) }
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> LayoutNode<'dom>
    for ServoLayoutNode<'dom, LayoutDataType>
{
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'dom, LayoutDataType>;

    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode {
        ServoThreadSafeLayoutNode::new(*self)
    }

    fn type_id(&self) -> LayoutNodeType {
        self.script_type_id().into()
    }

    unsafe fn initialize_data(&self) {
        if self.get_style_and_opaque_layout_data().is_none() {
            let opaque = StyleAndOpaqueLayoutData::new(
                StyleData::new(),
                AtomicRefCell::new(LayoutDataType::default()),
            );
            self.init_style_and_opaque_layout_data(opaque);
        };
    }

    unsafe fn init_style_and_opaque_layout_data(&self, data: Box<StyleAndOpaqueLayoutData>) {
        self.get_jsmanaged().init_style_and_opaque_layout_data(data);
    }

    unsafe fn take_style_and_opaque_layout_data(&self) -> Box<StyleAndOpaqueLayoutData> {
        self.get_jsmanaged().take_style_and_opaque_layout_data()
    }

    fn is_connected(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_CONNECTED) }
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> GetStyleAndOpaqueLayoutData<'dom>
    for ServoLayoutNode<'dom, LayoutDataType>
{
    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        self.get_jsmanaged().get_style_and_opaque_layout_data()
    }
}

/// A wrapper around a `ServoLayoutNode` that can be used safely on different threads.
/// It's very important that this never mutate anything except this wrapped node and
/// never access any other node apart from its parent.
pub struct ServoThreadSafeLayoutNode<'dom, LayoutDataType: LayoutDataTrait> {
    /// The wrapped `ServoLayoutNode`.
    pub(super) node: ServoLayoutNode<'dom, LayoutDataType>,

    /// The pseudo-element type, with (optionally)
    /// a specified display value to override the stylesheet.
    pub(super) pseudo: PseudoElementType,
}

// These impls are required because `derive` has trouble with PhantomData.
// See https://github.com/rust-lang/rust/issues/52079
impl<'dom, LayoutDataType: LayoutDataTrait> Clone
    for ServoThreadSafeLayoutNode<'dom, LayoutDataType>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<'dom, LayoutDataType: LayoutDataTrait> Copy
    for ServoThreadSafeLayoutNode<'dom, LayoutDataType>
{
}
impl<'a, LayoutDataType: LayoutDataTrait> PartialEq
    for ServoThreadSafeLayoutNode<'a, LayoutDataType>
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<'lr, LayoutDataType: LayoutDataTrait> fmt::Debug
    for ServoThreadSafeLayoutNode<'lr, LayoutDataType>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> ServoThreadSafeLayoutNode<'dom, LayoutDataType> {
    /// Creates a new `ServoThreadSafeLayoutNode` from the given `ServoLayoutNode`.
    pub fn new(node: ServoLayoutNode<'dom, LayoutDataType>) -> Self {
        ServoThreadSafeLayoutNode {
            node: node.clone(),
            pseudo: PseudoElementType::Normal,
        }
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

impl<'dom, LayoutDataType: LayoutDataTrait> style::dom::NodeInfo
    for ServoThreadSafeLayoutNode<'dom, LayoutDataType>
{
    fn is_element(&self) -> bool {
        self.node.is_element()
    }

    fn is_text_node(&self) -> bool {
        self.node.is_text_node()
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> ThreadSafeLayoutNode<'dom>
    for ServoThreadSafeLayoutNode<'dom, LayoutDataType>
{
    type ConcreteNode = ServoLayoutNode<'dom, LayoutDataType>;
    type ConcreteThreadSafeLayoutElement = ServoThreadSafeLayoutElement<'dom, LayoutDataType>;
    type ConcreteElement = ServoLayoutElement<'dom, LayoutDataType>;
    type ChildrenIterator = ServoThreadSafeLayoutNodeChildrenIterator<'dom, LayoutDataType>;

    fn opaque(&self) -> style::dom::OpaqueNode {
        unsafe { self.get_jsmanaged().opaque() }
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        if self.pseudo == PseudoElementType::Normal {
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

    fn as_element(&self) -> Option<ServoThreadSafeLayoutElement<'dom, LayoutDataType>> {
        self.node
            .as_element()
            .map(|el| ServoThreadSafeLayoutElement {
                element: el,
                pseudo: self.pseudo,
            })
    }

    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        self.node.get_style_and_opaque_layout_data()
    }

    fn is_ignorable_whitespace(&self, context: &SharedStyleContext) -> bool {
        unsafe {
            let text: LayoutDom<Text> = match self.get_jsmanaged().downcast() {
                Some(text) => text,
                None => return false,
            };

            if !is_whitespace(text.upcast().data_for_layout()) {
                return false;
            }

            // NB: See the rules for `white-space` here:
            //
            //    http://www.w3.org/TR/CSS21/text.html#propdef-white-space
            //
            // If you implement other values for this property, you will almost certainly
            // want to update this check.
            !self
                .style(context)
                .get_inherited_text()
                .white_space
                .preserve_newlines()
        }
    }

    unsafe fn unsafe_get(self) -> Self::ConcreteNode {
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

    fn get_colspan(&self) -> u32 {
        unsafe {
            self.get_jsmanaged()
                .downcast::<Element>()
                .unwrap()
                .get_colspan()
        }
    }

    fn get_rowspan(&self) -> u32 {
        unsafe {
            self.get_jsmanaged()
                .downcast::<Element>()
                .unwrap()
                .get_rowspan()
        }
    }
}

pub struct ServoThreadSafeLayoutNodeChildrenIterator<'dom, LayoutDataType: LayoutDataTrait> {
    current_node: Option<ServoThreadSafeLayoutNode<'dom, LayoutDataType>>,
    parent_node: ServoThreadSafeLayoutNode<'dom, LayoutDataType>,
}

impl<'dom, LayoutDataType: LayoutDataTrait>
    ServoThreadSafeLayoutNodeChildrenIterator<'dom, LayoutDataType>
{
    pub fn new(parent: ServoThreadSafeLayoutNode<'dom, LayoutDataType>) -> Self {
        let first_child = match parent.get_pseudo_element_type() {
            PseudoElementType::Normal => parent
                .get_before_pseudo()
                .or_else(|| parent.get_details_summary_pseudo())
                .or_else(|| unsafe { parent.dangerous_first_child() }),
            PseudoElementType::DetailsContent | PseudoElementType::DetailsSummary => unsafe {
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

impl<'dom, LayoutDataType: LayoutDataTrait> Iterator
    for ServoThreadSafeLayoutNodeChildrenIterator<'dom, LayoutDataType>
{
    type Item = ServoThreadSafeLayoutNode<'dom, LayoutDataType>;
    fn next(&mut self) -> Option<ServoThreadSafeLayoutNode<'dom, LayoutDataType>> {
        use selectors::Element;
        match self.parent_node.get_pseudo_element_type() {
            PseudoElementType::Before | PseudoElementType::After => None,

            PseudoElementType::DetailsSummary => {
                let mut current_node = self.current_node.clone();
                loop {
                    let next_node = if let Some(ref node) = current_node {
                        if let Some(element) = node.as_element() {
                            if element.has_local_name(&local_name!("summary")) &&
                                element.has_namespace(&ns!(html))
                            {
                                self.current_node = None;
                                return Some(node.clone());
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

            PseudoElementType::DetailsContent => {
                let node = self.current_node.clone();
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

            PseudoElementType::Normal => {
                let node = self.current_node.clone();
                if let Some(ref node) = node {
                    self.current_node = match node.get_pseudo_element_type() {
                        PseudoElementType::Before => self
                            .parent_node
                            .get_details_summary_pseudo()
                            .or_else(|| unsafe { self.parent_node.dangerous_first_child() })
                            .or_else(|| self.parent_node.get_after_pseudo()),
                        PseudoElementType::Normal => unsafe { node.dangerous_next_sibling() }
                            .or_else(|| self.parent_node.get_after_pseudo()),
                        PseudoElementType::DetailsSummary => {
                            self.parent_node.get_details_content_pseudo()
                        },
                        PseudoElementType::DetailsContent => self.parent_node.get_after_pseudo(),
                        PseudoElementType::After => None,
                    };
                }
                node
            },
        }
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> GetStyleAndOpaqueLayoutData<'dom>
    for ServoThreadSafeLayoutNode<'dom, LayoutDataType>
{
    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        self.node.get_style_and_opaque_layout_data()
    }
}
