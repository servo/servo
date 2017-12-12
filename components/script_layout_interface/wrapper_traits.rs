/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use HTMLCanvasData;
use LayoutNodeType;
use OpaqueStyleAndLayoutData;
use SVGSVGData;
use atomic_refcell::AtomicRef;
use gfx_traits::{ByteIndex, FragmentType, combine_id_with_fragment_type};
use html5ever::{Namespace, LocalName};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use range::Range;
use servo_arc::Arc;
use servo_url::ServoUrl;
use std::fmt::Debug;
use style::attr::AttrValue;
use style::computed_values::display::T as Display;
use style::context::SharedStyleContext;
use style::data::ElementData;
use style::dom::{LayoutIterator, NodeInfo, TNode};
use style::dom::OpaqueNode;
use style::font_metrics::ServoMetricsProvider;
use style::properties::{CascadeFlags, ComputedValues};
use style::selector_parser::{PseudoElement, PseudoElementCascadeType, SelectorImpl};
use style::stylist::RuleInclusion;
use webrender_api::ClipId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PseudoElementType<T> {
    Normal,
    Before(T),
    After(T),
    DetailsSummary(T),
    DetailsContent(T),
}

impl<T> PseudoElementType<T> {
    pub fn is_before(&self) -> bool {
        match *self {
            PseudoElementType::Before(_) => true,
            _ => false,
        }
    }

    pub fn is_replaced_content(&self) -> bool {
        match *self {
            PseudoElementType::Before(_) | PseudoElementType::After(_) => true,
            _ => false,
        }
    }

    pub fn strip(&self) -> PseudoElementType<()> {
        match *self {
            PseudoElementType::Normal => PseudoElementType::Normal,
            PseudoElementType::Before(_) => PseudoElementType::Before(()),
            PseudoElementType::After(_) => PseudoElementType::After(()),
            PseudoElementType::DetailsSummary(_) => PseudoElementType::DetailsSummary(()),
            PseudoElementType::DetailsContent(_) => PseudoElementType::DetailsContent(()),
        }
    }

    pub fn style_pseudo_element(&self) -> PseudoElement {
        match *self {
            PseudoElementType::Normal => unreachable!("style_pseudo_element called with PseudoElementType::Normal"),
            PseudoElementType::Before(_) => PseudoElement::Before,
            PseudoElementType::After(_) => PseudoElement::After,
            PseudoElementType::DetailsSummary(_) => PseudoElement::DetailsSummary,
            PseudoElementType::DetailsContent(_) => PseudoElement::DetailsContent,
        }
    }
}

/// Trait to abstract access to layout data across various data structures.
pub trait GetLayoutData {
    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData>;
}

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `LayoutDom`.
pub trait LayoutNode: Debug + GetLayoutData + TNode {
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode;
    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode;

    /// Returns the type ID of this node.
    fn type_id(&self) -> LayoutNodeType;

    unsafe fn initialize_data(&self);
    unsafe fn init_style_and_layout_data(&self, data: OpaqueStyleAndLayoutData);
    unsafe fn take_style_and_layout_data(&self) -> OpaqueStyleAndLayoutData;

    fn rev_children(self) -> LayoutIterator<ReverseChildrenIterator<Self>> {
        LayoutIterator(ReverseChildrenIterator {
            current: self.last_child(),
        })
    }

    fn traverse_preorder(self) -> TreeIterator<Self> {
        TreeIterator::new(self)
    }
}

pub struct ReverseChildrenIterator<ConcreteNode> where ConcreteNode: LayoutNode {
    current: Option<ConcreteNode>,
}

impl<ConcreteNode> Iterator for ReverseChildrenIterator<ConcreteNode>
                            where ConcreteNode: LayoutNode {
    type Item = ConcreteNode;
    fn next(&mut self) -> Option<ConcreteNode> {
        let node = self.current;
        self.current = node.and_then(|node| node.prev_sibling());
        node
    }
}

pub struct TreeIterator<ConcreteNode> where ConcreteNode: LayoutNode {
    stack: Vec<ConcreteNode>,
}

impl<ConcreteNode> TreeIterator<ConcreteNode> where ConcreteNode: LayoutNode {
    fn new(root: ConcreteNode) -> TreeIterator<ConcreteNode> {
        let mut stack = vec![];
        stack.push(root);
        TreeIterator {
            stack: stack,
        }
    }

    pub fn next_skipping_children(&mut self) -> Option<ConcreteNode> {
        self.stack.pop()
    }
}

impl<ConcreteNode> Iterator for TreeIterator<ConcreteNode>
                            where ConcreteNode: LayoutNode {
    type Item = ConcreteNode;
    fn next(&mut self) -> Option<ConcreteNode> {
        let ret = self.stack.pop();
        ret.map(|node| self.stack.extend(node.rev_children()));
        ret
    }
}


/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.
pub trait ThreadSafeLayoutNode: Clone + Copy + Debug + GetLayoutData + NodeInfo + PartialEq + Sized {
    type ConcreteNode: LayoutNode<ConcreteThreadSafeLayoutNode = Self>;
    type ConcreteThreadSafeLayoutElement:
        ThreadSafeLayoutElement<ConcreteThreadSafeLayoutNode = Self>
        + ::selectors::Element<Impl=SelectorImpl>;
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
        self.as_element().and_then(|el| el.get_before_pseudo()).map(|el| el.as_node())
    }

    fn get_after_pseudo(&self) -> Option<Self> {
        self.as_element().and_then(|el| el.get_after_pseudo()).map(|el| el.as_node())
    }

    fn get_details_summary_pseudo(&self) -> Option<Self> {
        self.as_element().and_then(|el| el.get_details_summary_pseudo()).map(|el| el.as_node())
    }

    fn get_details_content_pseudo(&self) -> Option<Self> {
        self.as_element().and_then(|el| el.get_details_content_pseudo()).map(|el| el.as_node())
    }

    fn debug_id(self) -> usize;

    /// Returns an iterator over this node's children.
    fn children(&self) -> LayoutIterator<Self::ChildrenIterator>;

    /// Returns a ThreadSafeLayoutElement if this is an element, None otherwise.
    #[inline]
    fn as_element(&self) -> Option<Self::ConcreteThreadSafeLayoutElement>;

    #[inline]
    fn get_pseudo_element_type(&self) -> PseudoElementType<Option<Display>> {
        self.as_element().map_or(PseudoElementType::Normal, |el| el.get_pseudo_element_type())
    }

    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData>;

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
    unsafe fn unsafe_get(self) -> Self::ConcreteNode;

    fn can_be_fragmented(&self) -> bool;

    fn node_text_content(&self) -> String;

    /// If the insertion point is within this node, returns it. Otherwise, returns `None`.
    fn selection(&self) -> Option<Range<ByteIndex>>;

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    fn image_url(&self) -> Option<ServoUrl>;

    fn canvas_data(&self) -> Option<HTMLCanvasData>;

    fn svg_data(&self) -> Option<SVGSVGData>;

    /// If this node is an iframe element, returns its browsing context ID. If this node is
    /// not an iframe element, fails. Returns None if there is no nested browsing context.
    fn iframe_browsing_context_id(&self) -> Option<BrowsingContextId>;

    /// If this node is an iframe element, returns its pipeline ID. If this node is
    /// not an iframe element, fails. Returns None if there is no nested browsing context.
    fn iframe_pipeline_id(&self) -> Option<PipelineId>;

    fn get_colspan(&self) -> u32;

    fn get_rowspan(&self) -> u32;

    fn fragment_type(&self) -> FragmentType {
        match self.get_pseudo_element_type() {
            PseudoElementType::Normal => FragmentType::FragmentBody,
            PseudoElementType::Before(_) => FragmentType::BeforePseudoContent,
            PseudoElementType::After(_) => FragmentType::AfterPseudoContent,
            PseudoElementType::DetailsSummary(_) => FragmentType::FragmentBody,
            PseudoElementType::DetailsContent(_) => FragmentType::FragmentBody,
        }
    }

    fn generate_scroll_root_id(&self, pipeline_id: PipelineId) -> ClipId {
        let id = combine_id_with_fragment_type(self.opaque().id(), self.fragment_type());
        ClipId::new(id as u64, pipeline_id.to_webrender())
    }
}

// This trait is only public so that it can be implemented by the gecko wrapper.
// It can be used to violate thread-safety, so don't use it elsewhere in layout!
#[allow(unsafe_code)]
pub trait DangerousThreadSafeLayoutNode: ThreadSafeLayoutNode {
    unsafe fn dangerous_first_child(&self) -> Option<Self>;
    unsafe fn dangerous_next_sibling(&self) -> Option<Self>;
}

pub trait ThreadSafeLayoutElement
    : Clone
    + Copy
    + Sized
    + Debug
    + ::selectors::Element<Impl=SelectorImpl>
    + GetLayoutData
{
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<ConcreteThreadSafeLayoutElement = Self>;

    fn as_node(&self) -> Self::ConcreteThreadSafeLayoutNode;

    /// Creates a new `ThreadSafeLayoutElement` for the same `LayoutElement`
    /// with a different pseudo-element type.
    fn with_pseudo(&self, pseudo: PseudoElementType<Option<Display>>) -> Self;

    /// Returns the type ID of this node.
    /// Returns `None` if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<LayoutNodeType>;

    /// Returns access to the underlying TElement. This is breaks the abstraction
    /// barrier of ThreadSafeLayout wrapper layer, and can lead to races if not used
    /// carefully.
    ///
    /// We need this so that the functions defined on this trait can call
    /// lazily_compute_pseudo_element_style, which operates on TElement.
    unsafe fn unsafe_get(self) ->
        <<Self::ConcreteThreadSafeLayoutNode as ThreadSafeLayoutNode>::ConcreteNode as TNode>::ConcreteElement;

    #[inline]
    fn get_attr(&self, namespace: &Namespace, name: &LocalName) -> Option<&str>;

    fn get_attr_enum(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue>;

    fn style_data(&self) -> AtomicRef<ElementData>;

    #[inline]
    fn get_pseudo_element_type(&self) -> PseudoElementType<Option<Display>>;

    #[inline]
    fn get_before_pseudo(&self) -> Option<Self> {
        if self.style_data().styles.pseudos.get(&PseudoElement::Before).is_some() {
            Some(self.with_pseudo(PseudoElementType::Before(None)))
        } else {
            None
        }
    }

    #[inline]
    fn get_after_pseudo(&self) -> Option<Self> {
        if self.style_data().styles.pseudos.get(&PseudoElement::After).is_some() {
            Some(self.with_pseudo(PseudoElementType::After(None)))
        } else {
            None
        }
    }

    #[inline]
    fn get_details_summary_pseudo(&self) -> Option<Self> {
        if self.get_local_name() == &local_name!("details") &&
           self.get_namespace() == &ns!(html) {
            Some(self.with_pseudo(PseudoElementType::DetailsSummary(None)))
        } else {
            None
        }
    }

    #[inline]
    fn get_details_content_pseudo(&self) -> Option<Self> {
        if self.get_local_name() == &local_name!("details") &&
           self.get_namespace() == &ns!(html) {
            let display = if self.get_attr(&ns!(), &local_name!("open")).is_some() {
                None // Specified by the stylesheet
            } else {
                Some(Display::None)
            };
            Some(self.with_pseudo(PseudoElementType::DetailsContent(display)))
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
            PseudoElementType::Normal => {
                data.styles.primary().clone()
            },
            other => {
                // Precompute non-eagerly-cascaded pseudo-element styles if not
                // cached before.
                let style_pseudo = other.style_pseudo_element();
                match style_pseudo.cascade_type() {
                    // Already computed during the cascade.
                    PseudoElementCascadeType::Eager => {
                        self.style_data()
                            .styles.pseudos.get(&style_pseudo)
                            .unwrap().clone()
                    },
                    PseudoElementCascadeType::Precomputed => {
                        context.stylist.precomputed_values_for_pseudo(
                            &context.guards,
                            &style_pseudo,
                            Some(data.styles.primary()),
                            CascadeFlags::empty(),
                            &ServoMetricsProvider)
                            .clone()
                    }
                    PseudoElementCascadeType::Lazy => {
                        context.stylist
                               .lazily_compute_pseudo_element_style(
                                   &context.guards,
                                   unsafe { &self.unsafe_get() },
                                   &style_pseudo,
                                   RuleInclusion::All,
                                   data.styles.primary(),
                                   /* is_probe = */ false,
                                   &ServoMetricsProvider,
                                   /* matching_func = */ None)
                               .unwrap()
                               .clone()
                    }
                }
            }
        }
    }

    #[inline]
    fn selected_style(&self) -> Arc<ComputedValues> {
        let data = self.style_data();
        data.styles.pseudos
            .get(&PseudoElement::Selection).map(|s| s)
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
            PseudoElementType::Normal
                => data.styles.primary().clone(),
            other
                => data.styles.pseudos
                       .get(&other.style_pseudo_element()).unwrap().clone(),
        }
    }
}
