/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use HTMLCanvasData;
use LayoutNodeType;
use OpaqueStyleAndLayoutData;
use PartialStyleAndLayoutData;
use gfx_traits::{ByteIndex, LayerId, LayerType};
use msg::constellation_msg::PipelineId;
use range::Range;
use restyle_damage::RestyleDamage;
use std::sync::Arc;
use string_cache::{Atom, BorrowedAtom, BorrowedNamespace, Namespace};
use style::computed_values::display;
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use style::dom::{PresentationalHintsSynthetizer, TNode};
use style::properties::ServoComputedValues;
use style::refcell::{Ref, RefCell};
use style::selector_impl::{PseudoElement, PseudoElementCascadeType, ServoSelectorImpl};
use url::Url;

#[derive(Copy, PartialEq, Clone)]
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

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `LayoutJS`.
pub trait LayoutNode: TNode {
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode;
    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode;

    /// Returns the type ID of this node.
    fn type_id(&self) -> LayoutNodeType;

    fn get_style_data(&self) -> Option<&RefCell<PartialStyleAndLayoutData>>;

    fn init_style_and_layout_data(&self, data: OpaqueStyleAndLayoutData);
    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData>;
}

/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.
pub trait ThreadSafeLayoutNode: Clone + Copy + Sized + PartialEq {
    type ConcreteThreadSafeLayoutElement:
        ThreadSafeLayoutElement<ConcreteThreadSafeLayoutNode = Self>
        + ::selectors::Element<Impl=ServoSelectorImpl, AttrString=String>;
    type ChildrenIterator: Iterator<Item = Self> + Sized;

    /// Creates a new `ThreadSafeLayoutNode` for the same `LayoutNode`
    /// with a different pseudo-element type.
    fn with_pseudo(&self, pseudo: PseudoElementType<Option<display::T>>) -> Self;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// Returns the type ID of this node.
    /// Returns `None` if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<LayoutNodeType>;

    /// Returns the type ID of this node, without discarding pseudo-elements as
    /// `type_id` does.
    fn type_id_without_excluding_pseudo_elements(&self) -> LayoutNodeType;

    #[inline]
    fn is_element_or_elements_pseudo(&self) -> bool {
        match self.type_id_without_excluding_pseudo_elements() {
            LayoutNodeType::Element(..) => true,
            _ => false,
        }
    }

    fn debug_id(self) -> usize;

    /// Returns an iterator over this node's children.
    fn children(&self) -> Self::ChildrenIterator;

    #[inline]
    fn is_element(&self) -> bool { if let Some(LayoutNodeType::Element(_)) = self.type_id() { true } else { false } }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    fn as_element(&self) -> Self::ConcreteThreadSafeLayoutElement;

    #[inline]
    fn get_pseudo_element_type(&self) -> PseudoElementType<Option<display::T>>;

    #[inline]
    fn get_before_pseudo(&self) -> Option<Self> {
        if self.get_style_data()
               .unwrap()
               .borrow()
               .style_data
               .per_pseudo
               .contains_key(&PseudoElement::Before) {
            Some(self.with_pseudo(PseudoElementType::Before(None)))
        } else {
            None
        }
    }

    #[inline]
    fn get_after_pseudo(&self) -> Option<Self> {
        if self.get_style_data()
               .unwrap()
               .borrow()
               .style_data
               .per_pseudo
               .contains_key(&PseudoElement::After) {
            Some(self.with_pseudo(PseudoElementType::After(None)))
        } else {
            None
        }
    }

    #[inline]
    fn get_details_summary_pseudo(&self) -> Option<Self> {
        if self.is_element() &&
           self.as_element().get_local_name() == atom!("details") &&
           self.as_element().get_namespace() == ns!(html) {
            Some(self.with_pseudo(PseudoElementType::DetailsSummary(None)))
        } else {
            None
        }
    }

    #[inline]
    fn get_details_content_pseudo(&self) -> Option<Self> {
        if self.is_element() &&
           self.as_element().get_local_name() == atom!("details") &&
           self.as_element().get_namespace() == ns!(html) {
            let display = if self.as_element().get_attr(&ns!(), &atom!("open")).is_some() {
                None // Specified by the stylesheet
            } else {
                Some(display::T::none)
            };
            Some(self.with_pseudo(PseudoElementType::DetailsContent(display)))
        } else {
            None
        }
    }

    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData>;

    /// Returns the style results for the given node. If CSS selector matching
    /// has not yet been performed, fails.
    ///
    /// Unlike the version on TNode, this handles pseudo-elements.
    #[inline]
    fn style(&self, context: &SharedStyleContext) -> Ref<Arc<ServoComputedValues>> {
        match self.get_pseudo_element_type() {
            PseudoElementType::Normal => {
                Ref::map(self.get_style_data().unwrap().borrow(), |data| {
                    data.style_data.style.as_ref().unwrap()
                })
            },
            other => {
                // Precompute non-eagerly-cascaded pseudo-element styles if not
                // cached before.
                let style_pseudo = other.style_pseudo_element();
                match style_pseudo.cascade_type() {
                    // Already computed during the cascade.
                    PseudoElementCascadeType::Eager => {},
                    PseudoElementCascadeType::Precomputed => {
                        if !self.get_style_data()
                                .unwrap()
                                .borrow()
                                .style_data
                                .per_pseudo.contains_key(&style_pseudo) {
                            let mut data = self.get_style_data().unwrap().borrow_mut();
                            let new_style =
                                context.stylist
                                       .precomputed_values_for_pseudo(&style_pseudo,
                                                                      data.style_data.style.as_ref());
                            data.style_data.per_pseudo
                                .insert(style_pseudo.clone(), new_style.unwrap());
                        }
                    }
                    PseudoElementCascadeType::Lazy => {
                        debug_assert!(self.is_element_or_elements_pseudo());
                        if !self.get_style_data()
                                .unwrap()
                                .borrow()
                                .style_data
                                .per_pseudo.contains_key(&style_pseudo) {
                            let mut data = self.get_style_data().unwrap().borrow_mut();
                            let new_style =
                                context.stylist
                                       .lazily_compute_pseudo_element_style(
                                           &self.as_element(),
                                           &style_pseudo,
                                           data.style_data.style.as_ref().unwrap());
                            data.style_data.per_pseudo
                                .insert(style_pseudo.clone(), new_style.unwrap());
                        }
                    }
                }

                Ref::map(self.get_style_data().unwrap().borrow(), |data| {
                    data.style_data.per_pseudo.get(&style_pseudo).unwrap()
                })
            }
        }
    }

    /// Returns the already resolved style of the node.
    ///
    /// This differs from `style(ctx)` in that if the pseudo-element has not yet
    /// been computed it would panic.
    ///
    /// This should be used just for querying layout, or when we know the
    /// element style is precomputed, not from general layout itself.
    #[inline]
    fn resolved_style(&self) -> Ref<Arc<ServoComputedValues>> {
        Ref::map(self.get_style_data().unwrap().borrow(), |data| {
            match self.get_pseudo_element_type() {
                PseudoElementType::Normal
                    => data.style_data.style.as_ref().unwrap(),
                other
                    => data.style_data.per_pseudo.get(&other.style_pseudo_element()).unwrap(),
            }
        })
    }

    #[inline]
    fn selected_style(&self, _context: &SharedStyleContext) -> Ref<Arc<ServoComputedValues>> {
        Ref::map(self.get_style_data().unwrap().borrow(), |data| {
            data.style_data.per_pseudo
                .get(&PseudoElement::Selection)
                .unwrap_or(data.style_data.style.as_ref().unwrap())
        })
    }

    /// Removes the style from this node.
    ///
    /// Unlike the version on TNode, this handles pseudo-elements.
    fn unstyle(self) {
        let mut data = self.get_style_data().unwrap().borrow_mut();

        match self.get_pseudo_element_type() {
            PseudoElementType::Normal => {
                data.style_data.style = None;
            }
            other => {
                data.style_data.per_pseudo.remove(&other.style_pseudo_element());
            }
        };
    }

    fn is_ignorable_whitespace(&self, context: &SharedStyleContext) -> bool;

    fn restyle_damage(self) -> RestyleDamage;

    fn set_restyle_damage(self, damage: RestyleDamage);

    /// Returns true if this node contributes content. This is used in the implementation of
    /// `empty_cells` per CSS 2.1 § 17.6.1.1.
    fn is_content(&self) -> bool {
        match self.type_id() {
            Some(LayoutNodeType::Element(..)) | Some(LayoutNodeType::Text) => true,
            _ => false
        }
    }

    fn can_be_fragmented(&self) -> bool;

    fn node_text_content(&self) -> String;

    /// If the insertion point is within this node, returns it. Otherwise, returns `None`.
    fn selection(&self) -> Option<Range<ByteIndex>>;

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    ///
    /// FIXME(pcwalton): Don't copy URLs.
    fn image_url(&self) -> Option<Url>;

    fn canvas_data(&self) -> Option<HTMLCanvasData>;

    /// If this node is an iframe element, returns its pipeline ID. If this node is
    /// not an iframe element, fails.
    fn iframe_pipeline_id(&self) -> PipelineId;

    fn get_colspan(&self) -> u32;

    fn layer_id(&self) -> LayerId {
        let layer_type = match self.get_pseudo_element_type() {
            PseudoElementType::Normal => LayerType::FragmentBody,
            PseudoElementType::Before(_) => LayerType::BeforePseudoContent,
            PseudoElementType::After(_) => LayerType::AfterPseudoContent,
            PseudoElementType::DetailsSummary(_) => LayerType::FragmentBody,
            PseudoElementType::DetailsContent(_) => LayerType::FragmentBody,
        };
        LayerId::new_of_type(layer_type, self.opaque().id() as usize)
    }

    fn layer_id_for_overflow_scroll(&self) -> LayerId {
        LayerId::new_of_type(LayerType::OverflowScroll, self.opaque().id() as usize)
    }

    fn get_style_data(&self) -> Option<&RefCell<PartialStyleAndLayoutData>>;
}

// This trait is only public so that it can be implemented by the gecko wrapper.
// It can be used to violate thread-safety, so don't use it elsewhere in layout!
#[allow(unsafe_code)]
pub trait DangerousThreadSafeLayoutNode: ThreadSafeLayoutNode {
    unsafe fn dangerous_first_child(&self) -> Option<Self>;
    unsafe fn dangerous_next_sibling(&self) -> Option<Self>;
}

pub trait ThreadSafeLayoutElement: Clone + Copy + Sized +
                                   ::selectors::Element<Impl=ServoSelectorImpl, AttrString=String> +
                                   PresentationalHintsSynthetizer {
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<ConcreteThreadSafeLayoutElement = Self>;

    #[inline]
    fn get_attr(&self, namespace: &Namespace, name: &Atom) -> Option<&str>;

    #[inline]
    fn get_local_name<'a>(&'a self) -> BorrowedAtom<'a>;

    #[inline]
    fn get_namespace<'a>(&'a self) -> BorrowedNamespace<'a>;
}
