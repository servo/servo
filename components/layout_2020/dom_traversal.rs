/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use html5ever::{local_name, LocalName};
use log::warn;
use script_layout_interface::wrapper_traits::{ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use script_layout_interface::{LayoutElementType, LayoutNodeType};
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::generics::counters::{Content, ContentItem};

use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox, NodeExt};
use crate::fragment_tree::{BaseFragmentInfo, FragmentFlags, Tag};
use crate::replaced::ReplacedContent;
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside, DisplayOutside};

#[derive(Clone, Copy, Debug)]
pub(crate) enum WhichPseudoElement {
    Before,
    After,
}

/// A data structure used to pass and store related layout information together to
/// avoid having to repeat the same arguments in argument lists.
#[derive(Clone)]
pub(crate) struct NodeAndStyleInfo<Node> {
    pub node: Option<Node>,
    pub pseudo_element_type: Option<WhichPseudoElement>,
    pub style: ServoArc<ComputedValues>,
}

impl<'dom, Node: NodeExt<'dom>> NodeAndStyleInfo<Node> {
    fn new_with_pseudo(
        node: Node,
        pseudo_element_type: WhichPseudoElement,
        style: ServoArc<ComputedValues>,
    ) -> Self {
        Self {
            node: Some(node),
            pseudo_element_type: Some(pseudo_element_type),
            style,
        }
    }

    pub(crate) fn new(node: Node, style: ServoArc<ComputedValues>) -> Self {
        Self {
            node: Some(node),
            pseudo_element_type: None,
            style,
        }
    }

    pub(crate) fn is_single_line_text_input(&self) -> bool {
        self.node.map_or(false, |node| {
            node.type_id() == LayoutNodeType::Element(LayoutElementType::HTMLInputElement)
        })
    }
}

impl<Node: Clone> NodeAndStyleInfo<Node> {
    pub(crate) fn new_anonymous(&self, style: ServoArc<ComputedValues>) -> Self {
        Self {
            node: None,
            pseudo_element_type: self.pseudo_element_type,
            style,
        }
    }

    pub(crate) fn new_replacing_style(&self, style: ServoArc<ComputedValues>) -> Self {
        Self {
            node: self.node.clone(),
            pseudo_element_type: self.pseudo_element_type,
            style,
        }
    }
}

impl<'dom, Node> From<&NodeAndStyleInfo<Node>> for BaseFragmentInfo
where
    Node: NodeExt<'dom>,
{
    fn from(info: &NodeAndStyleInfo<Node>) -> Self {
        let node = match info.node {
            Some(node) => node,
            None => return Self::anonymous(),
        };

        let pseudo = info.pseudo_element_type.map(|pseudo| match pseudo {
            WhichPseudoElement::Before => PseudoElement::Before,
            WhichPseudoElement::After => PseudoElement::After,
        });

        let threadsafe_node = node.to_threadsafe();
        let mut flags = FragmentFlags::empty();

        if let Some(element) = threadsafe_node.as_html_element() {
            if element.is_body_element_of_html_element_root() {
                flags.insert(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT);
            }
            match element.get_local_name() {
                &local_name!("br") => {
                    flags.insert(FragmentFlags::IS_BR_ELEMENT);
                },
                &local_name!("table") | &local_name!("th") | &local_name!("td") => {
                    flags.insert(FragmentFlags::IS_TABLE_TH_OR_TD_ELEMENT);
                },
                _ => {},
            }
        };

        Self {
            tag: Some(Tag::new_pseudo(threadsafe_node.opaque(), pseudo)),
            flags,
        }
    }
}

#[derive(Debug)]
pub(super) enum Contents {
    /// Any kind of content that is not replaced, including the contents of pseudo-elements.
    NonReplaced(NonReplacedContents),
    /// Example: an `<img src=…>` element.
    /// <https://drafts.csswg.org/css2/conform.html#replaced-element>
    Replaced(ReplacedContent),
}

#[derive(Debug)]
pub(super) enum NonReplacedContents {
    /// Refers to a DOM subtree, plus `::before` and `::after` pseudo-elements.
    OfElement,
    /// Content of a `::before` or `::after` pseudo-element that is being generated.
    /// <https://drafts.csswg.org/css2/generate.html#content>
    OfPseudoElement(Vec<PseudoElementContentItem>),
}

#[derive(Debug)]
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
        if child.is_text_node() {
            let info = NodeAndStyleInfo::new(child, child.style(context));
            handler.handle_text(&info, child.to_threadsafe().node_text_content());
        } else if child.is_element() {
            traverse_element(child, context, handler);
        }
    }

    if matches!(
        parent_element.type_id(),
        LayoutNodeType::Element(LayoutElementType::HTMLInputElement)
    ) {
        let info = NodeAndStyleInfo::new(parent_element, parent_element.style(context));

        // The addition of zero-width space here forces the text input to have an inline formatting
        // context that might otherwise be trimmed if there's no text. This is important to ensure
        // that the input element is at least as tall as the line gap of the caret:
        // <https://drafts.csswg.org/css-ui/#element-with-default-preferred-size>.
        //
        // TODO: Is there a less hacky way to do this?
        handler.handle_text(&info, "\u{200B}".into());

        handler.handle_text(&info, parent_element.to_threadsafe().node_text_content());
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
    let replaced = ReplacedContent::for_element(element, context);
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
            let contents =
                replaced.map_or(NonReplacedContents::OfElement.into(), Contents::Replaced);
            let display = display.used_value_for_contents(&contents);
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
                let contents = NonReplacedContents::OfPseudoElement(items).into();
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
            PseudoElementContentItem::Text(text) => handler.handle_text(info, text.into()),
            PseudoElementContentItem::Replaced(contents) => {
                let item_style = anonymous_style.get_or_insert_with(|| {
                    context
                        .shared_context()
                        .stylist
                        .style_for_anonymous::<Node::ConcreteElement>(
                            &context.shared_context().guards,
                            &PseudoElement::ServoAnonymousBox,
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
        matches!(self, Contents::Replaced(_))
    }
}

impl From<NonReplacedContents> for Contents {
    fn from(non_replaced_contents: NonReplacedContents) -> Self {
        Contents::NonReplaced(non_replaced_contents)
    }
}

impl std::convert::TryFrom<Contents> for NonReplacedContents {
    type Error = &'static str;

    fn try_from(contents: Contents) -> Result<Self, Self::Error> {
        match contents {
            Contents::NonReplaced(non_replaced_contents) => Ok(non_replaced_contents),
            Contents::Replaced(_) => {
                Err("Tried to covnert a `Contents::Replaced` into `NonReplacedContent`")
            },
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
        let node = match info.node {
            Some(node) => node,
            None => {
                warn!("Tried to traverse an anonymous node!");
                return;
            },
        };
        match self {
            NonReplacedContents::OfElement => traverse_children_of(node, context, handler),
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

/// <https://www.w3.org/TR/CSS2/generate.html#propdef-content>
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
            for item in items.items.iter() {
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
                    ContentItem::Image(image) => {
                        if let Some(replaced_content) =
                            ReplacedContent::from_image(element, context, image)
                        {
                            vec.push(PseudoElementContentItem::Replaced(replaced_content));
                        }
                    },
                    ContentItem::Counter(_, _) |
                    ContentItem::Counters(_, _, _) |
                    ContentItem::OpenQuote |
                    ContentItem::CloseQuote |
                    ContentItem::NoOpenQuote |
                    ContentItem::NoCloseQuote => {
                        // TODO: Add support for counters and quotes.
                    },
                }
            }
            vec
        },
        Content::Normal | Content::None => unreachable!(),
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
