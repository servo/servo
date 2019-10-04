/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::element_data::{LayoutBox, LayoutDataForElement};
use crate::replaced::ReplacedContent;
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside, DisplayOutside};
use crate::wrapper::GetRawData;
use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use servo_arc::Arc;
use std::convert::TryInto;
use std::marker::PhantomData as marker;
use style::context::SharedStyleContext;
use style::dom::TNode;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;

#[derive(Clone, Copy)]
pub enum WhichPseudoElement {
    Before,
    After,
}

pub(super) enum Contents<Node> {
    /// Refers to a DOM subtree, plus `::before` and `::after` pseudo-elements.
    OfElement(Node),

    /// Example: an `<img src=…>` element.
    /// <https://drafts.csswg.org/css2/conform.html#replaced-element>
    Replaced(ReplacedContent),

    /// Content of a `::before` or `::after` pseudo-element this is being generated.
    /// <https://drafts.csswg.org/css2/generate.html#content>
    OfPseudoElement(Vec<PseudoElementContentItem>),
}

pub(super) enum NonReplacedContents<Node> {
    OfElement(Node),
    OfPseudoElement(Vec<PseudoElementContentItem>),
}

pub(super) enum PseudoElementContentItem {
    Text(String),
    Replaced(ReplacedContent),
}

pub(super) trait TraversalHandler<'dom, Node>
where
    Node: 'dom,
{
    fn handle_text(&mut self, text: String, parent_style: &Arc<ComputedValues>);

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        style: &Arc<ComputedValues>,
        display: DisplayGeneratingBox,
        contents: Contents<Node>,
        box_slot: BoxSlot<'dom>,
    );
}

fn traverse_children_of<'dom, Node>(
    parent_element: Node,
    context: &SharedStyleContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
) where
    Node: NodeExt<'dom>,
{
    traverse_pseudo_element(WhichPseudoElement::Before, parent_element, context, handler);

    let mut next = parent_element.first_child();
    while let Some(child) = next {
        if let Some(contents) = child.as_text() {
            handler.handle_text(contents, &child.style(context));
        } else if child.is_element() {
            traverse_element(child, context, handler);
        }
        next = child.next_sibling();
    }

    traverse_pseudo_element(WhichPseudoElement::After, parent_element, context, handler);
}

fn traverse_element<'dom, Node>(
    element: Node,
    context: &SharedStyleContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
) where
    Node: NodeExt<'dom>,
{
    let style = element.style(context);
    match Display::from(style.get_box().display) {
        Display::None => element.unset_boxes_in_subtree(),
        Display::Contents => {
            if ReplacedContent::for_element(element, context).is_some() {
                // `display: content` on a replaced element computes to `display: none`
                // <https://drafts.csswg.org/css-display-3/#valdef-display-contents>
                element.unset_boxes_in_subtree()
            } else {
                *element.layout_data_mut().self_box.borrow_mut() = Some(LayoutBox::DisplayContents);
                traverse_children_of(element, context, handler)
            }
        },
        Display::GeneratingBox(display) => {
            handler.handle_element(
                &style,
                display,
                match ReplacedContent::for_element(element, context) {
                    Some(replaced) => Contents::Replaced(replaced),
                    None => Contents::OfElement(element),
                },
                element.element_box_slot(),
            );
        },
    }
}

fn traverse_pseudo_element<'dom, Node>(
    which: WhichPseudoElement,
    element: Node,
    context: &SharedStyleContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
) where
    Node: NodeExt<'dom>,
{
    if let Some(style) = pseudo_element_style(which, element, context) {
        match Display::from(style.get_box().display) {
            Display::None => element.unset_pseudo_element_box(which),
            Display::Contents => {
                element.unset_pseudo_element_box(which);
                let items = generate_pseudo_element_content(&style, element, context);
                traverse_pseudo_element_contents(&style, context, handler, items);
            },
            Display::GeneratingBox(display) => {
                let items = generate_pseudo_element_content(&style, element, context);
                let contents = Contents::OfPseudoElement(items);
                let box_slot = element.pseudo_element_box_slot(which);
                handler.handle_element(&style, display, contents, box_slot);
            },
        }
    }
}

fn traverse_pseudo_element_contents<'dom, Node>(
    pseudo_element_style: &Arc<ComputedValues>,
    context: &SharedStyleContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
    items: Vec<PseudoElementContentItem>,
) where
    Node: NodeExt<'dom>,
{
    let mut anonymous_style = None;
    for item in items {
        match item {
            PseudoElementContentItem::Text(text) => handler.handle_text(text, pseudo_element_style),
            PseudoElementContentItem::Replaced(contents) => {
                let item_style = anonymous_style.get_or_insert_with(|| {
                    context
                        .stylist
                        .style_for_anonymous::<Node::ConcreteElement>(
                            &context.guards,
                            &PseudoElement::ServoText,
                            &pseudo_element_style,
                        )
                });
                let display_inline = DisplayGeneratingBox::OutsideInside {
                    outside: DisplayOutside::Inline,
                    inside: DisplayInside::Flow,
                };
                // `display` is not inherited, so we get the initial value
                debug_assert!(
                    Display::from(item_style.get_box().display) ==
                        Display::GeneratingBox(display_inline)
                );
                handler.handle_element(
                    item_style,
                    display_inline,
                    Contents::Replaced(contents),
                    // We don’t keep pointers to boxes generated by contents of pseudo-elements
                    BoxSlot::dummy(),
                )
            },
        }
    }
}

impl<Node> std::convert::TryFrom<Contents<Node>> for NonReplacedContents<Node> {
    type Error = ReplacedContent;

    fn try_from(contents: Contents<Node>) -> Result<Self, Self::Error> {
        match contents {
            Contents::OfElement(node) => Ok(NonReplacedContents::OfElement(node)),
            Contents::OfPseudoElement(items) => Ok(NonReplacedContents::OfPseudoElement(items)),
            Contents::Replaced(replaced) => Err(replaced),
        }
    }
}

impl<Node> std::convert::From<NonReplacedContents<Node>> for Contents<Node> {
    fn from(contents: NonReplacedContents<Node>) -> Self {
        match contents {
            NonReplacedContents::OfElement(node) => Contents::OfElement(node),
            NonReplacedContents::OfPseudoElement(items) => Contents::OfPseudoElement(items),
        }
    }
}

impl<'dom, Node> NonReplacedContents<Node>
where
    Node: NodeExt<'dom>,
{
    pub(crate) fn traverse(
        self,
        inherited_style: &Arc<ComputedValues>,
        context: &SharedStyleContext,
        handler: &mut impl TraversalHandler<'dom, Node>,
    ) {
        match self {
            NonReplacedContents::OfElement(node) => traverse_children_of(node, context, handler),
            NonReplacedContents::OfPseudoElement(items) => {
                traverse_pseudo_element_contents(inherited_style, context, handler, items)
            },
        }
    }
}

fn pseudo_element_style<'dom, Node>(
    _which: WhichPseudoElement,
    _element: Node,
    _context: &SharedStyleContext,
) -> Option<Arc<ComputedValues>>
where
    Node: NodeExt<'dom>,
{
    // FIXME: run the cascade, then return None for `content: normal` or `content: none`
    // https://drafts.csswg.org/css2/generate.html#content
    None
}

fn generate_pseudo_element_content<'dom, Node>(
    _pseudo_element_style: &ComputedValues,
    _element: Node,
    _context: &SharedStyleContext,
) -> Vec<PseudoElementContentItem>
where
    Node: NodeExt<'dom>,
{
    let _ = PseudoElementContentItem::Text;
    let _ = PseudoElementContentItem::Replaced;
    unimplemented!()
}

pub struct BoxSlot<'dom> {
    slot: Option<Arc<AtomicRefCell<Option<LayoutBox>>>>,
    marker: marker<&'dom ()>,
}

impl BoxSlot<'_> {
    pub(crate) fn new(slot: Arc<AtomicRefCell<Option<LayoutBox>>>) -> Self {
        *slot.borrow_mut() = None;
        let slot = Some(slot);
        Self { slot, marker }
    }

    pub(crate) fn dummy() -> Self {
        let slot = None;
        Self { slot, marker }
    }

    pub(crate) fn set(mut self, box_: LayoutBox) {
        if let Some(slot) = &mut self.slot {
            *slot.borrow_mut() = Some(box_);
        }
    }
}

impl Drop for BoxSlot<'_> {
    fn drop(&mut self) {
        if let Some(slot) = &mut self.slot {
            assert!(slot.borrow().is_some(), "failed to set a layout box");
        }
    }
}

pub trait NodeExt<'dom>: 'dom + Copy + LayoutNode + Send + Sync {
    fn is_element(self) -> bool;
    fn as_text(self) -> Option<String>;
    fn first_child(self) -> Option<Self>;
    fn next_sibling(self) -> Option<Self>;
    fn parent_node(self) -> Option<Self>;
    fn style(self, context: &SharedStyleContext) -> Arc<ComputedValues>;

    fn layout_data_mut(&self) -> AtomicRefMut<LayoutDataForElement>;
    fn element_box_slot(&self) -> BoxSlot<'dom>;
    fn pseudo_element_box_slot(&self, which: WhichPseudoElement) -> BoxSlot<'dom>;
    fn unset_pseudo_element_box(self, which: WhichPseudoElement);
    fn unset_boxes_in_subtree(self);
}

impl<'dom, T> NodeExt<'dom> for T
where
    T: 'dom + Copy + LayoutNode + Send + Sync,
{
    fn is_element(self) -> bool {
        self.to_threadsafe().as_element().is_some()
    }

    fn as_text(self) -> Option<String> {
        if self.is_text_node() {
            Some(self.to_threadsafe().node_text_content())
        } else {
            None
        }
    }

    fn first_child(self) -> Option<Self> {
        TNode::first_child(&self)
    }

    fn next_sibling(self) -> Option<Self> {
        TNode::next_sibling(&self)
    }

    fn parent_node(self) -> Option<Self> {
        TNode::next_sibling(&self)
    }

    fn style(self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        self.to_threadsafe().style(context)
    }

    fn layout_data_mut(&self) -> AtomicRefMut<LayoutDataForElement> {
        self.get_raw_data()
            .map(|d| d.layout_data.borrow_mut())
            .unwrap()
    }

    fn element_box_slot(&self) -> BoxSlot<'dom> {
        BoxSlot::new(self.layout_data_mut().self_box.clone())
    }

    fn pseudo_element_box_slot(&self, which: WhichPseudoElement) -> BoxSlot<'dom> {
        let mut data = self.layout_data_mut();
        let pseudos = data.pseudo_elements.get_or_insert_with(Default::default);
        let cell = match which {
            WhichPseudoElement::Before => &mut pseudos.before,
            WhichPseudoElement::After => &mut pseudos.after,
        };
        BoxSlot::new(cell.clone())
    }

    fn unset_pseudo_element_box(self, which: WhichPseudoElement) {
        if let Some(pseudos) = &mut self.layout_data_mut().pseudo_elements {
            match which {
                WhichPseudoElement::Before => *pseudos.before.borrow_mut() = None,
                WhichPseudoElement::After => *pseudos.after.borrow_mut() = None,
            }
        }
    }

    fn unset_boxes_in_subtree(self) {
        let mut node = self;
        loop {
            if node.is_element() {
                let traverse_children = {
                    let mut layout_data = node.layout_data_mut();
                    layout_data.pseudo_elements = None;
                    let self_box = layout_data.self_box.borrow_mut().take();
                    self_box.is_some()
                };
                if traverse_children {
                    // Only descend into children if we removed a box.
                    // If there wasn’t one, then descendants don’t have boxes either.
                    if let Some(child) = node.first_child() {
                        node = child;
                        continue;
                    }
                }
            }
            let mut next_is_a_sibling_of = node;
            node = loop {
                if let Some(sibling) = next_is_a_sibling_of.next_sibling() {
                    break sibling;
                } else {
                    next_is_a_sibling_of = node
                        .parent_node()
                        .expect("reached the root while traversing only a subtree");
                }
            };
            if next_is_a_sibling_of == self {
                // Don’t go outside the subtree.
                return;
            }
        }
    }
}
