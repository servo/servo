/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::element_data::{LayoutBox, LayoutDataForElement};
use crate::geom::PhysicalSize;
use crate::replaced::{CanvasInfo, CanvasSource, ReplacedContent};
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside, DisplayOutside};
use crate::wrapper::GetRawData;
use atomic_refcell::AtomicRefMut;
use html5ever::LocalName;
use net_traits::image::base::Image as NetImage;
use script_layout_interface::wrapper_traits::{
    LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_layout_interface::HTMLCanvasDataSource;
use servo_arc::Arc as ServoArc;
use std::marker::PhantomData as marker;
use std::sync::{Arc, Mutex};
use style::dom::{OpaqueNode, TNode};
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::generics::counters::Content;
use style::values::generics::counters::ContentItem;

#[derive(Clone, Copy, Debug)]
pub enum WhichPseudoElement {
    Before,
    After,
}

pub(super) enum Contents {
    /// Refers to a DOM subtree, plus `::before` and `::after` pseudo-elements.
    OfElement,

    /// Example: an `<img src=…>` element.
    /// <https://drafts.csswg.org/css2/conform.html#replaced-element>
    Replaced(ReplacedContent),

    /// Content of a `::before` or `::after` pseudo-element that is being generated.
    /// <https://drafts.csswg.org/css2/generate.html#content>
    OfPseudoElement(Vec<PseudoElementContentItem>),
}

pub(super) enum NonReplacedContents {
    OfElement,
    OfPseudoElement(Vec<PseudoElementContentItem>),
}

pub(super) enum PseudoElementContentItem {
    Text(String),
    #[allow(dead_code)]
    Replaced(ReplacedContent),
}

pub(super) trait TraversalHandler<'dom, Node>
where
    Node: 'dom,
{
    fn handle_text(&mut self, node: Node, text: String, parent_style: &ServoArc<ComputedValues>);

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        node: Node,
        style: &ServoArc<ComputedValues>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    );
}

fn traverse_children_of<'dom, Node>(
    parent_element: Node,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
) where
    Node: NodeExt<'dom>,
{
    traverse_pseudo_element(WhichPseudoElement::Before, parent_element, context, handler);

    let mut next = parent_element.first_child();
    while let Some(child) = next {
        if let Some(contents) = child.as_text() {
            handler.handle_text(child, contents, &child.style(context));
        } else if child.is_element() {
            traverse_element(child, context, handler);
        }
        next = child.next_sibling();
    }

    traverse_pseudo_element(WhichPseudoElement::After, parent_element, context, handler);
}

fn traverse_element<'dom, Node>(
    element: Node,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
) where
    Node: NodeExt<'dom>,
{
    let replaced = ReplacedContent::for_element(element);
    let style = element.style(context);
    match Display::from(style.get_box().display) {
        Display::None => element.unset_boxes_in_subtree(),
        Display::Contents => {
            if replaced.is_some() {
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
                element,
                &style,
                display,
                replaced.map_or(Contents::OfElement, Contents::Replaced),
                element.element_box_slot(),
            );
        },
    }
}

fn traverse_pseudo_element<'dom, Node>(
    which: WhichPseudoElement,
    element: Node,
    context: &LayoutContext,
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
                traverse_pseudo_element_contents(element, &style, context, handler, items);
            },
            Display::GeneratingBox(display) => {
                let items = generate_pseudo_element_content(&style, element, context);
                let contents = Contents::OfPseudoElement(items);
                let box_slot = element.pseudo_element_box_slot(which);
                handler.handle_element(element, &style, display, contents, box_slot);
            },
        }
    }
}

fn traverse_pseudo_element_contents<'dom, Node>(
    node: Node,
    pseudo_element_style: &ServoArc<ComputedValues>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
    items: Vec<PseudoElementContentItem>,
) where
    Node: NodeExt<'dom>,
{
    let mut anonymous_style = None;
    for item in items {
        match item {
            PseudoElementContentItem::Text(text) => {
                handler.handle_text(node, text, pseudo_element_style)
            },
            PseudoElementContentItem::Replaced(contents) => {
                let item_style = anonymous_style.get_or_insert_with(|| {
                    context
                        .shared_context()
                        .stylist
                        .style_for_anonymous::<Node::ConcreteElement>(
                            &context.shared_context().guards,
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
                    node,
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

impl Contents {
    /// Returns true iff the `try_from` impl below would return `Err(_)`
    pub fn is_replaced(&self) -> bool {
        match self {
            Contents::OfElement | Contents::OfPseudoElement(_) => false,
            Contents::Replaced(_) => true,
        }
    }
}

impl std::convert::TryFrom<Contents> for NonReplacedContents {
    type Error = ReplacedContent;

    fn try_from(contents: Contents) -> Result<Self, Self::Error> {
        match contents {
            Contents::OfElement => Ok(NonReplacedContents::OfElement),
            Contents::OfPseudoElement(items) => Ok(NonReplacedContents::OfPseudoElement(items)),
            Contents::Replaced(replaced) => Err(replaced),
        }
    }
}

impl From<NonReplacedContents> for Contents {
    fn from(contents: NonReplacedContents) -> Self {
        match contents {
            NonReplacedContents::OfElement => Contents::OfElement,
            NonReplacedContents::OfPseudoElement(items) => Contents::OfPseudoElement(items),
        }
    }
}

impl NonReplacedContents {
    pub(crate) fn traverse<'dom, Node>(
        self,
        context: &LayoutContext,
        node: Node,
        inherited_style: &ServoArc<ComputedValues>,
        handler: &mut impl TraversalHandler<'dom, Node>,
    ) where
        Node: NodeExt<'dom>,
    {
        match self {
            NonReplacedContents::OfElement => traverse_children_of(node, context, handler),
            NonReplacedContents::OfPseudoElement(items) => {
                traverse_pseudo_element_contents(node, inherited_style, context, handler, items)
            },
        }
    }
}

fn pseudo_element_style<'dom, Node>(
    which: WhichPseudoElement,
    element: Node,
    context: &LayoutContext,
) -> Option<ServoArc<ComputedValues>>
where
    Node: NodeExt<'dom>,
{
    match which {
        WhichPseudoElement::Before => element.to_threadsafe().get_before_pseudo(),
        WhichPseudoElement::After => element.to_threadsafe().get_after_pseudo(),
    }
    .and_then(|pseudo_element| {
        let style = pseudo_element.style(context.shared_context());
        if style.ineffective_content_property() {
            None
        } else {
            Some(style)
        }
    })
}

/// https://www.w3.org/TR/CSS2/generate.html#propdef-content
fn generate_pseudo_element_content<'dom, Node>(
    pseudo_element_style: &ComputedValues,
    element: Node,
    context: &LayoutContext,
) -> Vec<PseudoElementContentItem>
where
    Node: NodeExt<'dom>,
{
    match &pseudo_element_style.get_counters().content {
        Content::Items(ref items) => {
            let mut vec = vec![];
            for item in items.iter() {
                match item {
                    ContentItem::String(s) => {
                        vec.push(PseudoElementContentItem::Text(s.to_string()));
                    },
                    ContentItem::Attr(attr) => {
                        let element = element
                            .to_threadsafe()
                            .as_element()
                            .expect("Expected an element");
                        let attr_val = element
                            .get_attr(&attr.namespace_url, &LocalName::from(&*attr.attribute));
                        vec.push(PseudoElementContentItem::Text(
                            attr_val.map_or("".to_string(), |s| s.to_string()),
                        ));
                    },
                    ContentItem::Url(image_url) => {
                        if let Some(replaced_content) =
                            ReplacedContent::from_image_url(element, context, image_url)
                        {
                            vec.push(PseudoElementContentItem::Replaced(replaced_content));
                        }
                    },
                    _ => (),
                }
            }
            vec
        },
        Content::Normal | Content::None => unreachable!(),
    }
}

pub struct BoxSlot<'dom> {
    slot: Option<ArcRefCell<Option<LayoutBox>>>,
    marker: marker<&'dom ()>,
}

impl BoxSlot<'_> {
    pub(crate) fn new(slot: ArcRefCell<Option<LayoutBox>>) -> Self {
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

pub(crate) trait NodeExt<'dom>: 'dom + Copy + LayoutNode + Send + Sync {
    fn is_element(self) -> bool;
    fn as_text(self) -> Option<String>;

    /// Returns the image if it’s loaded, and its size in image pixels
    /// adjusted for `image_density`.
    fn as_image(self) -> Option<(Option<Arc<NetImage>>, PhysicalSize<f64>)>;
    fn as_canvas(self) -> Option<(CanvasInfo, PhysicalSize<f64>)>;
    fn first_child(self) -> Option<Self>;
    fn next_sibling(self) -> Option<Self>;
    fn parent_node(self) -> Option<Self>;
    fn style(self, context: &LayoutContext) -> ServoArc<ComputedValues>;

    fn as_opaque(self) -> OpaqueNode;
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

    fn as_image(self) -> Option<(Option<Arc<NetImage>>, PhysicalSize<f64>)> {
        let node = self.to_threadsafe();
        let (resource, metadata) = node.image_data()?;
        let (width, height) = resource
            .as_ref()
            .map(|image| (image.width, image.height))
            .or_else(|| metadata.map(|metadata| (metadata.width, metadata.height)))
            .unwrap_or((0, 0));
        let (mut width, mut height) = (width as f64, height as f64);
        if let Some(density) = node.image_density().filter(|density| *density != 1.) {
            width = width / density;
            height = height / density;
        }
        Some((resource, PhysicalSize::new(width, height)))
    }

    fn as_canvas(self) -> Option<(CanvasInfo, PhysicalSize<f64>)> {
        let node = self.to_threadsafe();
        let canvas_data = node.canvas_data()?;
        let source = match canvas_data.source {
            HTMLCanvasDataSource::WebGL(texture_id) => CanvasSource::WebGL(texture_id),
            HTMLCanvasDataSource::Image(ipc_sender) => {
                CanvasSource::Image(ipc_sender.map(|renderer| Arc::new(Mutex::new(renderer))))
            },
        };
        Some((
            CanvasInfo {
                source,
                canvas_id: canvas_data.canvas_id,
            },
            PhysicalSize::new(canvas_data.width.into(), canvas_data.height.into()),
        ))
    }

    fn first_child(self) -> Option<Self> {
        TNode::first_child(&self)
    }

    fn next_sibling(self) -> Option<Self> {
        TNode::next_sibling(&self)
    }

    fn parent_node(self) -> Option<Self> {
        TNode::parent_node(&self)
    }

    fn style(self, context: &LayoutContext) -> ServoArc<ComputedValues> {
        self.to_threadsafe().style(context.shared_context())
    }

    fn as_opaque(self) -> OpaqueNode {
        self.opaque()
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
        assert!(self.is_element());
        assert!(self.parent_node().is_some());

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
                } else if node == self {
                    // If this is the root of the subtree and we aren't descending
                    // into our children return now.
                    return;
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
