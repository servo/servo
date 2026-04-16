/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(unsafe_code)]
#![deny(missing_docs)]

use std::borrow::Cow;
use std::fmt::Debug;

use net_traits::image_cache::Image;
use pixels::ImageMetadata;
use servo_arc::Arc;
use servo_base::id::{BrowsingContextId, PipelineId};
use servo_url::ServoUrl;
use style::context::SharedStyleContext;
use style::dom::{LayoutIterator, NodeInfo, OpaqueNode, TNode};
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;

use crate::layout_element::{DangerousStyleElement, LayoutElement};
use crate::pseudo_element_chain::PseudoElementChain;
use crate::{
    GenericLayoutData, HTMLCanvasData, HTMLMediaData, LayoutDataTrait, LayoutNodeType,
    SVGElementData, SharedSelection,
};

/// A trait that exposes a DOM nodes to layout. Implementors of this trait must abide by certain
/// safety requirements. Layout will only ever access and mutate each node from a single thread
/// at a time, though children may be used in parallel from other threads. That is why this trait
/// does not allow access to parent nodes, as it would make it easy to cause race conditions and
/// memory errors.
///
/// Note that the related [`DangerousStyleNode`] trait *may* access parent nodes, which is why
/// that API is marked as `unsafe` here. In general [`DangerousStyleNode`] should only be used
/// when interfacing with the `stylo` and `selectors`.
pub trait LayoutNode<'dom>: Copy + Debug + NodeInfo + Send + Sync {
    /// The concrete implementation of [`DangerousStyleNode`] implemented in `script`.
    type ConcreteDangerousStyleNode: DangerousStyleNode<'dom>;
    /// The concrete implementation of [`DangerousStyleElement`] implemented in `script`.
    type ConcreteDangerousStyleElement: DangerousStyleElement<'dom>;
    /// The concrete implementation of [`ConcreteLayoutElement`] implemented in `script`.
    type ConcreteLayoutElement: LayoutElement<'dom>;
    /// The concrete implementation of [`ChildIterator`] implemented in `script`.
    type ChildIterator: Iterator<Item = Self> + Sized;

    /// Creates a new `LayoutNode` for the same `LayoutNode` with a different pseudo-element type.
    ///
    /// Returns `None` if this pseudo doesn't apply to the given element for one of
    /// the following reasons:
    ///
    ///  1. This node is not an element.
    ///  2. `pseudo` is eager and is not defined in the stylesheet. In this case, there
    ///     is not reason to process the pseudo element at all.
    ///  3. `pseudo` is for `::servo-details-content` and
    ///     it doesn't apply to this element, either because it isn't a details or is
    ///     in the wrong state.
    fn with_pseudo(&self, pseudo_element_type: PseudoElement) -> Option<Self>;

    /// Returns the [`PseudoElementChain`] for this [`LayoutElement`].
    fn pseudo_element_chain(&self) -> PseudoElementChain;

    /// Returns access to a version of this LayoutNode that can be used by stylo
    /// and selectors. This is dangerous as it allows more access to ancestors nodes
    /// than LayoutNode. This should *only* be used when handing a node to stylo
    /// or selectors.
    ///
    /// # Safety
    ///
    /// This should only ever be called from the main script thread. It is never
    /// okay to explicitly create a node for style while any layout worker threads
    /// are running.
    unsafe fn dangerous_style_node(self) -> Self::ConcreteDangerousStyleNode;

    /// Returns access to the DOM parent node of this node. This *does not* take
    /// into account shadow tree children and slottables. For that use
    /// [`Self::dangerous_flat_tree_parent`].
    ///
    /// # Safety
    ///
    /// This should only ever be called from the main script thread. It is never
    /// okay to explicitly access the parent node while any layout worker threads
    /// are running.
    unsafe fn dangerous_dom_parent(self) -> Option<Self>;

    /// Returns access to the flat tree parent node of this node. This takes
    /// into account shadow tree children and slottables. For that use
    /// [`Self::dangerous_flat_tree_parent`].
    ///
    /// # Safety
    ///
    /// This should only ever be called from the main script thread. It is never
    /// okay to explicitly access the parent node while any layout worker threads
    /// are running.
    unsafe fn dangerous_flat_tree_parent(self) -> Option<Self>;

    /// Get the layout data of this node, attempting to downcast it to the desired type.
    /// Returns None if there is no layout data or it isn't of the desired type.
    fn layout_data(&self) -> Option<&'dom GenericLayoutData>;

    /// Returns whether the node is connected.
    fn is_connected(&self) -> bool;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// Returns the type ID of this node. Returns `None` if this is a pseudo-element; otherwise,
    /// returns `Some`.
    fn type_id(&self) -> Option<LayoutNodeType>;

    /// Initialize this node with empty opaque layout data.
    fn initialize_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self);

    /// Returns an iterator over this node's children in the [flat tree]. This
    /// takes into account shadow tree children and slottables.
    ///
    /// [flat tree]: https://drafts.csswg.org/css-shadow-1/#flat-tree
    fn flat_tree_children(&self) -> LayoutIterator<Self::ChildIterator>;

    /// Returns an iterator over this node's children in the DOM. This
    /// *does not* take shadow roots and assigned slottables into account.
    /// For that use [`Self::flat_tree_children`].
    fn dom_children(&self) -> LayoutIterator<Self::ChildIterator>;

    /// Returns a [`LayoutElement`] if this is an element in the HTML namespace, None otherwise.
    fn as_html_element(&self) -> Option<Self::ConcreteLayoutElement>;

    /// Returns a [`LayoutElement`] if this is an element.
    fn as_element(&self) -> Option<Self::ConcreteLayoutElement>;

    /// Returns the computed style for the given node, properly handling pseudo-elements. For
    /// elements this returns their style and for other nodes, this returns the style of the parent
    /// element, if one exists.
    ///
    /// # Panics
    ///
    /// - Calling this method will panic it is an element has no style data, whether because
    ///   styling has not run yet or was not run for this element.
    /// - Calling this method will panic if it is a non-element node without a parent element.
    fn style(&self, context: &SharedStyleContext) -> Arc<ComputedValues>;

    /// Returns the style for a text node. This is computed on the fly from the
    /// parent style to avoid traversing text nodes in the style system.
    ///
    /// # Safety
    ///
    /// Note that this does require accessing the parent, which this interface
    /// technically forbids. But accessing the parent is only unsafe insofar as
    /// it can be used to reach siblings and cousins. A simple immutable borrow
    /// of the parent data is fine, since the bottom-up traversal will not process
    /// the parent until all the children have been processed.
    ///
    /// # Panics
    ///
    /// - Calling this method will panic if the parent element has no style data, whether
    ///   because styling has not run yet or was not run for this element.
    /// - Calling this method will panic if it is a non-element node without a parent element.
    fn parent_style(&self, context: &SharedStyleContext) -> Arc<ComputedValues>;

    /// Returns the computed `:selected` style for the given node, properly handling
    /// pseudo-elements. For elements this returns their style and for other nodes, this
    /// returns the style of the parent element, if one exists.
    ///
    /// # Panics
    ///
    /// - Calling this method will panic it is an element has no style data, whether because
    ///   styling has not run yet or was not run for this element.
    /// - Calling this method will panic if it is a non-element node without a parent element.
    fn selected_style(&self, context: &SharedStyleContext) -> Arc<ComputedValues>;

    /// Get the text content of this node, if it is a text node.
    ///
    /// # Panics
    ///
    /// This method will panic if called on a node that is not a DOM text node.
    fn text_content(self) -> Cow<'dom, str>;

    /// If this node manages a selection, this returns the shared selection for the node.
    fn selection(&self) -> Option<SharedSelection>;

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    fn image_url(&self) -> Option<ServoUrl>;

    /// If this is an image element, returns its current-pixel-density. If this is not an image element, fails.
    fn image_density(&self) -> Option<f64>;

    /// If this is an image element, returns its image data. Otherwise, returns `None`.
    fn image_data(&self) -> Option<(Option<Image>, Option<ImageMetadata>)>;

    /// Whether or not this is an image element that is showing a broken image icon.
    fn showing_broken_image_icon(&self) -> bool;

    /// Return the [`HTMLCanvas`] data for this node, if it is a canvas.
    fn canvas_data(&self) -> Option<HTMLCanvasData>;

    /// Return the [`SVGElementData`] for this node, if it is an SVG subtree.
    fn svg_data(&self) -> Option<SVGElementData<'dom>>;

    /// Return the [`HTMLMediaData`] for this node, if it is a media element.
    fn media_data(&self) -> Option<HTMLMediaData>;

    /// If this node is an iframe element, returns its browsing context ID. If this node is
    /// not an iframe element, fails. Returns None if there is no nested browsing context.
    fn iframe_browsing_context_id(&self) -> Option<BrowsingContextId>;

    /// If this node is an iframe element, returns its pipeline ID. If this node is
    /// not an iframe element, fails. Returns None if there is no nested browsing context.
    fn iframe_pipeline_id(&self) -> Option<PipelineId>;

    /// Return the table span property if it is an element that supports it.
    fn table_span(&self) -> Option<u32>;

    /// Return the table colspan property if it is an element that supports it.
    fn table_colspan(&self) -> Option<u32>;

    /// Return the table rowspan property if it is an element that supports it.
    fn table_rowspan(&self) -> Option<u32>;

    /// Whether this is a container for the text within a single-line text input. This
    /// is used to solve the special case of line height for a text entry widget.
    /// <https://html.spec.whatwg.org/multipage/#the-input-element-as-a-text-entry-widget>
    fn is_single_line_text_input(&self) -> bool;

    /// Whether or not this [`LayoutNode`] is in a user agent widget shadow DOM.
    fn is_root_of_user_agent_widget(&self) -> bool;

    /// Set whether or not this node has an active pseudo-element style with a `content`
    /// attribute that uses `attr`.
    fn set_uses_content_attribute_with_attr(&self, _uses_content_attribute_with_attr: bool);
}

/// A node that can be passed to `stylo` and `selectors` that allows accessing the
/// parent node. We consider this to be too dangerous for normal layout, so it is
/// reserved only for using `stylo` and `selectors`.
///
/// If you are not interfacing with `stylo` and `selectors` you *should not* use this
/// type, unless you know what you are doing.
pub trait DangerousStyleNode<'dom>: TNode + Sized + NodeInfo + Send + Sync {
    /// The concrete implementation of [`LayoutNode`] implemented in `script`.
    type ConcreteLayoutNode: LayoutNode<'dom>;
    /// Get a handle to the original "safe" version of this node, a [`LayoutNode`] implementation.
    fn layout_node(&self) -> Self::ConcreteLayoutNode;
}
