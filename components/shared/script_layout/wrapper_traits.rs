/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::fmt::Debug;

use atomic_refcell::AtomicRef;
use base::id::{BrowsingContextId, PipelineId};
use fonts_traits::ByteIndex;
use html5ever::{LocalName, Namespace};
use net_traits::image_cache::Image;
use pixels::ImageMetadata;
use range::Range;
use servo_arc::Arc;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::context::SharedStyleContext;
use style::data::ElementData;
use style::dom::{LayoutIterator, NodeInfo, OpaqueNode, TElement, TNode};
use style::properties::ComputedValues;
use style::selector_parser::{PseudoElement, PseudoElementCascadeType, SelectorImpl};
use style::stylist::RuleInclusion;

use crate::{
    FragmentType, GenericLayoutData, GenericLayoutDataTrait, HTMLCanvasData, HTMLMediaData,
    LayoutNodeType, SVGSVGData, StyleData,
};

pub trait LayoutDataTrait: GenericLayoutDataTrait + Default + Send + Sync + 'static {}

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `LayoutDom`.
/// FIXME(mrobinson): `Send + Sync` is required here for Layout 2020, but eventually it
/// should stop sending LayoutNodes to other threads and rely on ThreadSafeLayoutNode
/// or some other mechanism to ensure thread safety.
pub trait LayoutNode<'dom>: Copy + Debug + TNode + Send + Sync {
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<'dom>;
    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode;

    /// Returns the type ID of this node.
    fn type_id(&self) -> LayoutNodeType;

    /// Initialize this node with empty style and opaque layout data.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the given node during
    /// layout. Callers should ensure that no other layout thread is
    /// attempting to read or modify the opaque layout data of this node.
    unsafe fn initialize_style_and_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self);

    /// Initialize this node with empty opaque layout data.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the given node during
    /// layout. Callers should ensure that no other layout thread is
    /// attempting to read or modify the opaque layout data of this node.
    fn initialize_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self);

    /// Get the [`StyleData`] for this node. Returns None if the node is unstyled.
    fn style_data(&self) -> Option<&'dom StyleData>;

    /// Get the layout data of this node, attempting to downcast it to the desired type.
    /// Returns None if there is no layout data or it isn't of the desired type.
    fn layout_data(&self) -> Option<&'dom GenericLayoutData>;

    fn rev_children(self) -> LayoutIterator<ReverseChildrenIterator<Self>> {
        LayoutIterator(ReverseChildrenIterator {
            current: self.last_child(),
        })
    }

    fn traverse_preorder(self) -> TreeIterator<Self> {
        TreeIterator::new(self)
    }

    /// Returns whether the node is connected.
    fn is_connected(&self) -> bool;
}

pub struct ReverseChildrenIterator<ConcreteNode> {
    current: Option<ConcreteNode>,
}

impl<'dom, ConcreteNode> Iterator for ReverseChildrenIterator<ConcreteNode>
where
    ConcreteNode: LayoutNode<'dom>,
{
    type Item = ConcreteNode;
    fn next(&mut self) -> Option<ConcreteNode> {
        let node = self.current;
        self.current = node.and_then(|node| node.prev_sibling());
        node
    }
}

pub struct TreeIterator<ConcreteNode> {
    stack: Vec<ConcreteNode>,
}

impl<'dom, ConcreteNode> TreeIterator<ConcreteNode>
where
    ConcreteNode: LayoutNode<'dom>,
{
    fn new(root: ConcreteNode) -> TreeIterator<ConcreteNode> {
        let stack = vec![root];
        TreeIterator { stack }
    }

    pub fn next_skipping_children(&mut self) -> Option<ConcreteNode> {
        self.stack.pop()
    }
}

impl<'dom, ConcreteNode> Iterator for TreeIterator<ConcreteNode>
where
    ConcreteNode: LayoutNode<'dom>,
{
    type Item = ConcreteNode;
    fn next(&mut self) -> Option<ConcreteNode> {
        let ret = self.stack.pop();
        if let Some(node) = ret {
            self.stack.extend(node.rev_children())
        }
        ret
    }
}

/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.
pub trait ThreadSafeLayoutNode<'dom>: Clone + Copy + Debug + NodeInfo + PartialEq + Sized {
    type ConcreteNode: LayoutNode<'dom, ConcreteThreadSafeLayoutNode = Self>;
    type ConcreteElement: TElement;

    type ConcreteThreadSafeLayoutElement: ThreadSafeLayoutElement<'dom, ConcreteThreadSafeLayoutNode = Self>
        + ::selectors::Element<Impl = SelectorImpl>;
    type ChildrenIterator: Iterator<Item = Self> + Sized;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// Returns the type ID of this node.
    /// Returns `None` if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<LayoutNodeType>;

    /// Returns the style for a text node. This is computed on the fly from the
    /// parent style to avoid traversing text nodes in the style system.
    ///
    /// Note that this does require accessing the parent, which this interface
    /// technically forbids. But accessing the parent is only unsafe insofar as
    /// it can be used to reach siblings and cousins. A simple immutable borrow
    /// of the parent data is fine, since the bottom-up traversal will not process
    /// the parent until all the children have been processed.
    fn parent_style(&self) -> Arc<ComputedValues>;

    fn debug_id(self) -> usize;

    /// Returns an iterator over this node's children.
    fn children(&self) -> LayoutIterator<Self::ChildrenIterator>;

    /// Returns a ThreadSafeLayoutElement if this is an element, None otherwise.
    fn as_element(&self) -> Option<Self::ConcreteThreadSafeLayoutElement>;

    /// Returns a ThreadSafeLayoutElement if this is an element in an HTML namespace, None otherwise.
    fn as_html_element(&self) -> Option<Self::ConcreteThreadSafeLayoutElement>;

    /// Get the [`StyleData`] for this node. Returns None if the node is unstyled.
    fn style_data(&self) -> Option<&'dom StyleData>;

    /// Get the layout data of this node, attempting to downcast it to the desired type.
    /// Returns None if there is no layout data or it isn't of the desired type.
    fn layout_data(&self) -> Option<&'dom GenericLayoutData>;

    fn style(&self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        if let Some(el) = self.as_element() {
            el.style(context)
        } else {
            // Text nodes are not styled during traversal,instead we simply
            // return parent style here and do cascading during layout.
            debug_assert!(self.is_text_node());
            self.parent_style()
        }
    }

    fn selected_style(&self) -> Arc<ComputedValues> {
        if let Some(el) = self.as_element() {
            el.selected_style()
        } else {
            debug_assert!(self.is_text_node());
            // TODO(stshine): What should the selected style be for text?
            self.parent_style()
        }
    }

    /// Returns true if this node contributes content. This is used in the implementation of
    /// `empty_cells` per CSS 2.1 § 17.6.1.1.
    fn is_content(&self) -> bool {
        self.type_id().is_some()
    }

    /// Returns access to the underlying LayoutNode. This is breaks the abstraction
    /// barrier of ThreadSafeLayout wrapper layer, and can lead to races if not used
    /// carefully.
    ///
    /// We need this because the implementation of some methods need to access the layout
    /// data flags, and we have this annoying trait separation between script and layout :-(
    fn unsafe_get(self) -> Self::ConcreteNode;

    fn node_text_content(self) -> Cow<'dom, str>;

    /// If selection intersects this node, return it. Otherwise, returns `None`.
    fn selection(&self) -> Option<Range<ByteIndex>>;

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    fn image_url(&self) -> Option<ServoUrl>;

    /// If this is an image element, returns its current-pixel-density. If this is not an image element, fails.
    fn image_density(&self) -> Option<f64>;

    /// If this is an image element, returns its image data. Otherwise, returns `None`.
    fn image_data(&self) -> Option<(Option<Image>, Option<ImageMetadata>)>;

    fn canvas_data(&self) -> Option<HTMLCanvasData>;

    fn svg_data(&self) -> Option<SVGSVGData>;

    fn media_data(&self) -> Option<HTMLMediaData>;

    /// If this node is an iframe element, returns its browsing context ID. If this node is
    /// not an iframe element, fails. Returns None if there is no nested browsing context.
    fn iframe_browsing_context_id(&self) -> Option<BrowsingContextId>;

    /// If this node is an iframe element, returns its pipeline ID. If this node is
    /// not an iframe element, fails. Returns None if there is no nested browsing context.
    fn iframe_pipeline_id(&self) -> Option<PipelineId>;

    fn get_span(&self) -> Option<u32>;
    fn get_colspan(&self) -> Option<u32>;
    fn get_rowspan(&self) -> Option<u32>;

    fn pseudo_element(&self) -> Option<PseudoElement>;

    fn fragment_type(&self) -> FragmentType {
        self.pseudo_element().into()
    }

    fn with_pseudo(&self, pseudo_element_type: PseudoElement) -> Option<Self> {
        self.as_element()
            .and_then(|element| element.with_pseudo(pseudo_element_type))
            .as_ref()
            .map(ThreadSafeLayoutElement::as_node)
    }
}

pub trait ThreadSafeLayoutElement<'dom>:
    Clone + Copy + Sized + Debug + ::selectors::Element<Impl = SelectorImpl>
{
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<'dom, ConcreteThreadSafeLayoutElement = Self>;

    /// This type alias is just a work-around to avoid writing
    ///
    ///   <Self::ConcreteThreadSafeLayoutNode as ThreadSafeLayoutNode>::ConcreteElement
    ///
    type ConcreteElement: TElement;

    fn as_node(&self) -> Self::ConcreteThreadSafeLayoutNode;

    /// Creates a new `ThreadSafeLayoutElement` for the same `LayoutElement`
    /// with a different pseudo-element type.
    ///
    /// Returns `None` if this pseudo doesn't apply to the given element for one of
    /// the following reasons:
    ///
    ///  1. `pseudo` is eager and is not defined in the stylesheet. In this case, there
    ///     is not reason to process the pseudo element at all.
    ///  2. `pseudo` is for `::servo-details-summary` or `::servo-details-content` and
    ///     it doesn't apply to this element, either because it isn't a details or is
    ///     in the wrong state.
    fn with_pseudo(&self, pseudo: PseudoElement) -> Option<Self>;

    /// Returns the type ID of this node.
    /// Returns `None` if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<LayoutNodeType>;

    /// Returns access to the underlying TElement. This is breaks the abstraction
    /// barrier of ThreadSafeLayout wrapper layer, and can lead to races if not used
    /// carefully.
    ///
    /// We need this so that the functions defined on this trait can call
    /// lazily_compute_pseudo_element_style, which operates on TElement.
    fn unsafe_get(self) -> Self::ConcreteElement;

    /// Get the local name of this element. See
    /// <https://dom.spec.whatwg.org/#concept-element-local-name>.
    fn get_local_name(&self) -> &LocalName;

    fn get_attr(&self, namespace: &Namespace, name: &LocalName) -> Option<&str>;

    fn get_attr_enum(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue>;

    fn style_data(&self) -> AtomicRef<ElementData>;

    fn pseudo_element(&self) -> Option<PseudoElement>;

    /// Returns the style results for the given node. If CSS selector matching
    /// has not yet been performed, fails.
    ///
    /// Unlike the version on TNode, this handles pseudo-elements.
    #[inline]
    fn style(&self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        let data = self.style_data();
        match self.pseudo_element() {
            None => data.styles.primary().clone(),
            Some(style_pseudo) => {
                // Precompute non-eagerly-cascaded pseudo-element styles if not
                // cached before.
                match style_pseudo.cascade_type() {
                    // Already computed during the cascade.
                    PseudoElementCascadeType::Eager => self
                        .style_data()
                        .styles
                        .pseudos
                        .get(&style_pseudo)
                        .unwrap()
                        .clone(),
                    PseudoElementCascadeType::Precomputed => context
                        .stylist
                        .precomputed_values_for_pseudo::<Self::ConcreteElement>(
                            &context.guards,
                            &style_pseudo,
                            Some(data.styles.primary()),
                        ),
                    PseudoElementCascadeType::Lazy => {
                        context
                            .stylist
                            .lazily_compute_pseudo_element_style(
                                &context.guards,
                                self.unsafe_get(),
                                &style_pseudo,
                                RuleInclusion::All,
                                data.styles.primary(),
                                /* is_probe = */ false,
                                /* matching_func = */ None,
                            )
                            .unwrap()
                    },
                }
            },
        }
    }

    #[inline]
    fn selected_style(&self) -> Arc<ComputedValues> {
        let data = self.style_data();
        data.styles
            .pseudos
            .get(&PseudoElement::Selection)
            .unwrap_or(data.styles.primary())
            .clone()
    }

    fn is_shadow_host(&self) -> bool;

    /// Returns whether this node is a body element of an html element root
    /// in an HTML element document.
    ///
    /// Note that this does require accessing the parent, which this interface
    /// technically forbids. But accessing the parent is only unsafe insofar as
    /// it can be used to reach siblings and cousins. A simple immutable borrow
    /// of the parent data is fine, since the bottom-up traversal will not process
    /// the parent until all the children have been processed.
    fn is_body_element_of_html_element_root(&self) -> bool;

    /// Returns whether this node is the root element in an HTML document element.
    ///
    /// Note that, like `Self::is_body_element_of_html_element_root`, this accesses the parent.
    /// As in that case, since this is an immutable borrow, we do not violate thread safety.
    fn is_root(&self) -> bool;
}
