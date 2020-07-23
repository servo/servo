/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::element_data::{LayoutBox, LayoutDataForElement};
use crate::geom::PhysicalSize;
use crate::replaced::{CanvasInfo, CanvasSource, ReplacedContent};
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside, DisplayOutside};
use crate::wrapper::GetStyleAndLayoutData;
use atomic_refcell::AtomicRefMut;
use html5ever::LocalName;
use net_traits::image::base::Image as NetImage;
use script_layout_interface::wrapper_traits::{
    LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_layout_interface::HTMLCanvasDataSource;
use servo_arc::Arc as ServoArc;
use std::borrow::Cow;
use std::marker::PhantomData as marker;
use std::sync::{Arc, Mutex};
use style::dom::{OpaqueNode, TNode};
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::generics::counters::Content;
use style::values::generics::counters::ContentItem;

#[derive(Clone, Copy, Debug)]
pub(crate) enum WhichPseudoElement {
    Before,
    After,
}

/// A data structure used to pass and store related layout information together to
/// avoid having to repeat the same arguments in argument lists.
#[derive(Clone)]
pub(crate) struct NodeAndStyleInfo<Node> {
    pub node: Node,
    pub pseudo_element_type: Option<WhichPseudoElement>,
    pub style: ServoArc<ComputedValues>,
}

impl<Node> NodeAndStyleInfo<Node> {
    fn new_with_pseudo(
        node: Node,
        pseudo_element_type: WhichPseudoElement,
        style: ServoArc<ComputedValues>,
    ) -> Self {
        Self {
            node,
            pseudo_element_type: Some(pseudo_element_type),
            style,
        }
    }

    pub(crate) fn new(node: Node, style: ServoArc<ComputedValues>) -> Self {
        Self {
            node,
            pseudo_element_type: None,
            style,
        }
    }
}

impl<Node: Clone> NodeAndStyleInfo<Node> {
    pub(crate) fn new_replacing_style(&self, style: ServoArc<ComputedValues>) -> Self {
        Self {
            node: self.node.clone(),
            pseudo_element_type: self.pseudo_element_type.clone(),
            style,
        }
    }
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
    Replaced(ReplacedContent),
}

pub(super) trait TraversalHandler<'dom, Node>
where
    Node: 'dom,
{
    fn handle_text(&mut self, info: &NodeAndStyleInfo<Node>, text: Cow<'dom, str>);

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<Node>,
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

    for child in iter_child_nodes(parent_element) {
        if let Some(contents) = child.as_text() {
            let info = NodeAndStyleInfo::new(child, child.style(context));
            handler.handle_text(&info, contents);
        } else if child.is_element() {
            traverse_element(child, context, handler);
        }
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
        Display::None => element.unset_all_boxes(),
        Display::Contents => {
            if replaced.is_some() {
                // `display: content` on a replaced element computes to `display: none`
                // <https://drafts.csswg.org/css-display-3/#valdef-display-contents>
                element.unset_all_boxes()
            } else {
                element.element_box_slot().set(LayoutBox::DisplayContents);
                traverse_children_of(element, context, handler)
            }
        },
        Display::GeneratingBox(display) => {
            let contents = replaced.map_or(Contents::OfElement, Contents::Replaced);
            let box_slot = element.element_box_slot();
            let info = NodeAndStyleInfo::new(element, style);
            handler.handle_element(&info, display, contents, box_slot);
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
        let info = NodeAndStyleInfo::new_with_pseudo(element, which, style);
        match Display::from(info.style.get_box().display) {
            Display::None => element.unset_pseudo_element_box(which),
            Display::Contents => {
                let items = generate_pseudo_element_content(&info.style, element, context);
                let box_slot = element.pseudo_element_box_slot(which);
                box_slot.set(LayoutBox::DisplayContents);
                traverse_pseudo_element_contents(&info, context, handler, items);
            },
            Display::GeneratingBox(display) => {
                let items = generate_pseudo_element_content(&info.style, element, context);
                let box_slot = element.pseudo_element_box_slot(which);
                let contents = Contents::OfPseudoElement(items);
                handler.handle_element(&info, display, contents, box_slot);
            },
        }
    } else {
        element.unset_pseudo_element_box(which)
    }
}

fn traverse_pseudo_element_contents<'dom, Node>(
    info: &NodeAndStyleInfo<Node>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom, Node>,
    items: Vec<PseudoElementContentItem>,
) where
    Node: NodeExt<'dom>,
{
    let mut anonymous_style = None;
    for item in items {
        match item {
            PseudoElementContentItem::Text(text) => handler.handle_text(&info, text.into()),
            PseudoElementContentItem::Replaced(contents) => {
                let item_style = anonymous_style.get_or_insert_with(|| {
                    context
                        .shared_context()
                        .stylist
                        .style_for_anonymous::<Node::ConcreteElement>(
                            &context.shared_context().guards,
                            &PseudoElement::ServoText,
                            &info.style,
                        )
                });
                let display_inline = DisplayGeneratingBox::OutsideInside {
                    outside: DisplayOutside::Inline,
                    inside: DisplayInside::Flow {
                        is_list_item: false,
                    },
                };
                // `display` is not inherited, so we get the initial value
                debug_assert!(
                    Display::from(item_style.get_box().display) ==
                        Display::GeneratingBox(display_inline)
                );
                let info = info.new_replacing_style(item_style.clone());
                handler.handle_element(
                    &info,
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
        info: &NodeAndStyleInfo<Node>,
        handler: &mut impl TraversalHandler<'dom, Node>,
    ) where
        Node: NodeExt<'dom>,
    {
        match self {
            NonReplacedContents::OfElement => traverse_children_of(info.node, context, handler),
            NonReplacedContents::OfPseudoElement(items) => {
                traverse_pseudo_element_contents(info, context, handler, items)
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
        if !std::thread::panicking() {
            if let Some(slot) = &mut self.slot {
                assert!(slot.borrow().is_some(), "failed to set a layout box");
            }
        }
    }
}

pub(crate) trait NodeExt<'dom>: 'dom + Copy + LayoutNode<'dom> + Send + Sync {
    fn is_element(self) -> bool;
    fn as_text(self) -> Option<Cow<'dom, str>>;

    /// Returns the image if it’s loaded, and its size in image pixels
    /// adjusted for `image_density`.
    fn as_image(self) -> Option<(Option<Arc<NetImage>>, PhysicalSize<f64>)>;
    fn as_canvas(self) -> Option<(CanvasInfo, PhysicalSize<f64>)>;
    fn first_child(self) -> Option<Self>;
    fn next_sibling(self) -> Option<Self>;
    fn parent_node(self) -> Option<Self>;
    fn style(self, context: &LayoutContext) -> ServoArc<ComputedValues>;

    fn as_opaque(self) -> OpaqueNode;
    fn layout_data_mut(self) -> AtomicRefMut<'dom, LayoutDataForElement>;
    fn element_box_slot(&self) -> BoxSlot<'dom>;
    fn pseudo_element_box_slot(&self, which: WhichPseudoElement) -> BoxSlot<'dom>;
    fn unset_pseudo_element_box(self, which: WhichPseudoElement);

    /// Remove boxes for the element itself, and its `:before` and `:after` if any.
    fn unset_all_boxes(self);
}

impl<'dom, T> NodeExt<'dom> for T
where
    T: 'dom + Copy + LayoutNode<'dom> + Send + Sync,
{
    fn is_element(self) -> bool {
        self.to_threadsafe().as_element().is_some()
    }

    fn as_text(self) -> Option<Cow<'dom, str>> {
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
            HTMLCanvasDataSource::WebGPU(image_key) => CanvasSource::WebGPU(image_key),
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

    #[allow(unsafe_code)]
    fn layout_data_mut(self) -> AtomicRefMut<'dom, LayoutDataForElement> {
        self.get_style_and_layout_data()
            .map(|d| d.layout_data.borrow_mut())
            .unwrap()
    }

    fn element_box_slot(&self) -> BoxSlot<'dom> {
        BoxSlot::new(self.layout_data_mut().self_box.clone())
    }

    fn pseudo_element_box_slot(&self, which: WhichPseudoElement) -> BoxSlot<'dom> {
        let data = self.layout_data_mut();
        let cell = match which {
            WhichPseudoElement::Before => &data.pseudo_before_box,
            WhichPseudoElement::After => &data.pseudo_after_box,
        };
        BoxSlot::new(cell.clone())
    }

    fn unset_pseudo_element_box(self, which: WhichPseudoElement) {
        let data = self.layout_data_mut();
        let cell = match which {
            WhichPseudoElement::Before => &data.pseudo_before_box,
            WhichPseudoElement::After => &data.pseudo_after_box,
        };
        *cell.borrow_mut() = None;
    }

    fn unset_all_boxes(self) {
        let data = self.layout_data_mut();
        *data.self_box.borrow_mut() = None;
        *data.pseudo_before_box.borrow_mut() = None;
        *data.pseudo_after_box.borrow_mut() = None;
        // Stylo already takes care of removing all layout data
        // for DOM descendants of elements with `display: none`.
    }
}

pub(crate) fn iter_child_nodes<'dom, Node>(parent: Node) -> impl Iterator<Item = Node>
where
    Node: NodeExt<'dom>,
{
    let mut next = parent.first_child();
    std::iter::from_fn(move || {
        next.map(|child| {
            next = child.next_sibling();
            child
        })
    })
}
