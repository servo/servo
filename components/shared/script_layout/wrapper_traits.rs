/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc as StdArc;

use atomic_refcell::AtomicRef;
use base::id::{BrowsingContextId, PipelineId};
use fonts_traits::ByteIndex;
use html5ever::{local_name, namespace_url, ns, LocalName, Namespace};
use pixels::{Image, ImageMetadata};
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
    FragmentType, GenericLayoutData, HTMLCanvasData, HTMLMediaData, LayoutNodeType, SVGSVGData,
    StyleData,
};

pub trait LayoutDataTrait: Default + Send + Sync + 'static {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PseudoElementType {
    Normal,
    Before,
    After,
    DetailsSummary,
    DetailsContent,
}

impl PseudoElementType {
    pub fn fragment_type(&self) -> FragmentType {
        match *self {
            PseudoElementType::Normal => FragmentType::FragmentBody,
            PseudoElementType::Before => FragmentType::BeforePseudoContent,
            PseudoElementType::After => FragmentType::AfterPseudoContent,
            PseudoElementType::DetailsSummary => FragmentType::FragmentBody,
            PseudoElementType::DetailsContent => FragmentType::FragmentBody,
        }
    }

    pub fn is_before(&self) -> bool {
        matches!(*self, PseudoElementType::Before)
    }

    pub fn is_replaced_content(&self) -> bool {
        matches!(*self, PseudoElementType::Before | PseudoElementType::After)
    }

    pub fn style_pseudo_element(&self) -> PseudoElement {
        match *self {
            PseudoElementType::Normal => {
                unreachable!("style_pseudo_element called with PseudoElementType::Normal")
            },
            PseudoElementType::Before => PseudoElement::Before,
            PseudoElementType::After => PseudoElement::After,
            PseudoElementType::DetailsSummary => PseudoElement::DetailsSummary,
            PseudoElementType::DetailsContent => PseudoElement::DetailsContent,
        }
    }
}

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

    fn get_before_pseudo(&self) -> Option<Self> {
        self.as_element()
            .and_then(|el| el.get_before_pseudo())
            .map(|el| el.as_node())
    }

    fn get_after_pseudo(&self) -> Option<Self> {
        self.as_element()
            .and_then(|el| el.get_after_pseudo())
            .map(|el| el.as_node())
    }

    fn get_details_summary_pseudo(&self) -> Option<Self> {
        self.as_element()
            .and_then(|el| el.get_details_summary_pseudo())
            .map(|el| el.as_node())
    }

    fn get_details_content_pseudo(&self) -> Option<Self> {
        self.as_element()
            .and_then(|el| el.get_details_content_pseudo())
            .map(|el| el.as_node())
    }

    fn debug_id(self) -> usize;

    /// Returns an iterator over this node's children.
    fn children(&self) -> LayoutIterator<Self::ChildrenIterator>;

    /// Returns a ThreadSafeLayoutElement if this is an element, None otherwise.
    fn as_element(&self) -> Option<Self::ConcreteThreadSafeLayoutElement>;

    /// Returns a ThreadSafeLayoutElement if this is an element in an HTML namespace, None otherwise.
    fn as_html_element(&self) -> Option<Self::ConcreteThreadSafeLayoutElement>;

    #[inline]
    fn get_pseudo_element_type(&self) -> PseudoElementType {
        self.as_element()
            .map_or(PseudoElementType::Normal, |el| el.get_pseudo_element_type())
    }

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

    fn is_ignorable_whitespace(&self, context: &SharedStyleContext) -> bool;

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

    /// If the insertion point is within this node, returns it. Otherwise, returns `None`.
    fn selection(&self) -> Option<Range<ByteIndex>>;

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    fn image_url(&self) -> Option<ServoUrl>;

    /// If this is an image element, returns its current-pixel-density. If this is not an image element, fails.
    fn image_density(&self) -> Option<f64>;

    /// If this is an image element, returns its image data. Otherwise, returns `None`.
    fn image_data(&self) -> Option<(Option<StdArc<Image>>, Option<ImageMetadata>)>;

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

    fn fragment_type(&self) -> FragmentType {
        self.get_pseudo_element_type().fragment_type()
    }
}

pub trait ThreadSafeLayoutElement<'dom>:
    Clone + Copy + Sized + Debug + ::selectors::Element<Impl = SelectorImpl>
{
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<
        'dom,
        ConcreteThreadSafeLayoutElement = Self,
    >;

    /// This type alias is just a work-around to avoid writing
    ///
    ///   <Self::ConcreteThreadSafeLayoutNode as ThreadSafeLayoutNode>::ConcreteElement
    ///
    type ConcreteElement: TElement;

    fn as_node(&self) -> Self::ConcreteThreadSafeLayoutNode;

    /// Creates a new `ThreadSafeLayoutElement` for the same `LayoutElement`
    /// with a different pseudo-element type.
    fn with_pseudo(&self, pseudo: PseudoElementType) -> Self;

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

    fn get_pseudo_element_type(&self) -> PseudoElementType;

    #[inline]
    fn get_before_pseudo(&self) -> Option<Self> {
        if self
            .style_data()
            .styles
            .pseudos
            .get(&PseudoElement::Before)
            .is_some()
        {
            Some(self.with_pseudo(PseudoElementType::Before))
        } else {
            None
        }
    }

    #[inline]
    fn get_after_pseudo(&self) -> Option<Self> {
        if self
            .style_data()
            .styles
            .pseudos
            .get(&PseudoElement::After)
            .is_some()
        {
            Some(self.with_pseudo(PseudoElementType::After))
        } else {
            None
        }
    }

    #[inline]
    fn get_details_summary_pseudo(&self) -> Option<Self> {
        if self.has_local_name(&local_name!("details")) && self.has_namespace(&ns!(html)) {
            Some(self.with_pseudo(PseudoElementType::DetailsSummary))
        } else {
            None
        }
    }

    #[inline]
    fn get_details_content_pseudo(&self) -> Option<Self> {
        if self.has_local_name(&local_name!("details")) &&
            self.has_namespace(&ns!(html)) &&
            self.get_attr(&ns!(), &local_name!("open")).is_some()
        {
            Some(self.with_pseudo(PseudoElementType::DetailsContent))
        } else {
            None
        }
    }

    /// Returns the style results for the given node. If CSS selector matching
    /// has not yet been performed, fails.
    ///
    /// Unlike the version on TNode, this handles pseudo-elements.
    #[inline]
    fn style(&self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        let data = self.style_data();
        match self.get_pseudo_element_type() {
            PseudoElementType::Normal => data.styles.primary().clone(),
            other => {
                // Precompute non-eagerly-cascaded pseudo-element styles if not
                // cached before.
                let style_pseudo = other.style_pseudo_element();
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

    /// Returns the already resolved style of the node.
    ///
    /// This differs from `style(ctx)` in that if the pseudo-element has not yet
    /// been computed it would panic.
    ///
    /// This should be used just for querying layout, or when we know the
    /// element style is precomputed, not from general layout itself.
    #[inline]
    fn resolved_style(&self) -> Arc<ComputedValues> {
        let data = self.style_data();
        match self.get_pseudo_element_type() {
            PseudoElementType::Normal => data.styles.primary().clone(),
            other => data
                .styles
                .pseudos
                .get(&other.style_pseudo_element())
                .unwrap()
                .clone(),
        }
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
}
