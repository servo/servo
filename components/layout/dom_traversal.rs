/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::iter::FusedIterator;

use fonts::ByteIndex;
use html5ever::{LocalName, local_name};
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use layout_api::{LayoutDamage, LayoutElementType, LayoutNodeType};
use range::Range;
use script::layout_dom::ServoLayoutNode;
use selectors::Element as SelectorsElement;
use servo_arc::Arc as ServoArc;
use style::dom::{NodeInfo, TElement, TNode, TShadowRoot};
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::generics::counters::{Content, ContentItem};
use style::values::specified::Quotes;

use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox, NodeExt};
use crate::flow::inline::SharedInlineStyles;
use crate::fragment_tree::{BaseFragmentInfo, FragmentFlags, Tag};
use crate::quotes::quotes_for_lang;
use crate::replaced::ReplacedContents;
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside, DisplayOutside};

/// A data structure used to pass and store related layout information together to
/// avoid having to repeat the same arguments in argument lists.
#[derive(Clone)]
pub(crate) struct NodeAndStyleInfo<'dom> {
    pub node: ServoLayoutNode<'dom>,
    pub pseudo_element_type: Option<PseudoElement>,
    pub style: ServoArc<ComputedValues>,
    pub damage: LayoutDamage,
}

impl<'dom> NodeAndStyleInfo<'dom> {
    pub(crate) fn new(
        node: ServoLayoutNode<'dom>,
        style: ServoArc<ComputedValues>,
        damage: LayoutDamage,
    ) -> Self {
        Self {
            node,
            pseudo_element_type: None,
            style,
            damage,
        }
    }

    /// Whether this is a container for the editable text within a single-line text input.
    /// This is used to solve the special case of line height for a text editor.
    /// <https://html.spec.whatwg.org/multipage/#the-input-element-as-a-text-entry-widget>
    // FIXME(stevennovaryo): Now, this would also refer to HTMLInputElement, to handle input
    //                       elements that is yet to to be implemented with shadow DOM.
    pub(crate) fn is_single_line_text_input(&self) -> bool {
        self.node.type_id() == LayoutNodeType::Element(LayoutElementType::HTMLInputElement) ||
            self.node.is_single_line_text_inner_editor()
    }

    pub(crate) fn pseudo(
        &self,
        context: &LayoutContext,
        pseudo_element_type: PseudoElement,
    ) -> Option<Self> {
        let style = self
            .node
            .to_threadsafe()
            .as_element()?
            .with_pseudo(pseudo_element_type)?
            .style(&context.style_context);
        Some(NodeAndStyleInfo {
            node: self.node,
            pseudo_element_type: Some(pseudo_element_type),
            style,
            damage: self.damage,
        })
    }

    pub(crate) fn get_selected_style(&self) -> ServoArc<ComputedValues> {
        if self.node.is_single_line_text_inner_editor() {
            self.node
                .containing_shadow_host()
                .expect("Ua widget inner editor is not contained")
                .to_threadsafe()
                .selected_style()
        } else {
            self.node.to_threadsafe().selected_style()
        }
    }

    pub(crate) fn get_selection_range(&self) -> Option<Range<ByteIndex>> {
        self.node.to_threadsafe().selection()
    }
}

impl<'dom> From<&NodeAndStyleInfo<'dom>> for BaseFragmentInfo {
    fn from(info: &NodeAndStyleInfo<'dom>) -> Self {
        let node = info.node;
        let pseudo = info.pseudo_element_type;
        let threadsafe_node = node.to_threadsafe();
        let mut flags = FragmentFlags::empty();

        // Anonymous boxes should not have a tag, because they should not take part in hit testing.
        //
        // TODO(mrobinson): It seems that anonymous boxes should take part in hit testing in some
        // cases, but currently this means that the order of hit test results isn't as expected for
        // some WPT tests. This needs more investigation.
        if matches!(
            pseudo,
            Some(PseudoElement::ServoAnonymousBox) |
                Some(PseudoElement::ServoAnonymousTable) |
                Some(PseudoElement::ServoAnonymousTableCell) |
                Some(PseudoElement::ServoAnonymousTableRow)
        ) {
            return Self::anonymous();
        }

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

            if matches!(
                element.type_id(),
                Some(LayoutNodeType::Element(
                    LayoutElementType::HTMLInputElement | LayoutElementType::HTMLTextAreaElement
                ))
            ) {
                flags.insert(FragmentFlags::IS_TEXT_CONTROL);
            }

            if ThreadSafeLayoutElement::is_root(&element) {
                flags.insert(FragmentFlags::IS_ROOT_ELEMENT);
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
    Replaced(ReplacedContents),
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(super) enum NonReplacedContents {
    /// Refers to a DOM subtree, plus `::before` and `::after` pseudo-elements.
    OfElement,
    /// Content of a `::before` or `::after` pseudo-element that is being generated.
    /// <https://drafts.csswg.org/css2/generate.html#content>
    OfPseudoElement(Vec<PseudoElementContentItem>),
    /// Workaround for input and textarea element until we properly implement `display-inside`.
    OfTextControl,
}

#[derive(Debug)]
pub(super) enum PseudoElementContentItem {
    Text(String),
    Replaced(ReplacedContents),
}

pub(super) trait TraversalHandler<'dom> {
    fn handle_text(&mut self, info: &NodeAndStyleInfo<'dom>, text: Cow<'dom, str>);

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        info: &NodeAndStyleInfo<'dom>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    );

    /// Notify the handler that we are about to recurse into a `display: contents` element.
    fn enter_display_contents(&mut self, _: SharedInlineStyles) {}

    /// Notify the handler that we have finished a `display: contents` element.
    fn leave_display_contents(&mut self) {}
}

fn traverse_children_of<'dom>(
    parent_element_info: &NodeAndStyleInfo<'dom>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom>,
) {
    traverse_eager_pseudo_element(PseudoElement::Before, parent_element_info, context, handler);

    if parent_element_info.node.is_text_input() {
        let node_text_content = parent_element_info.node.to_threadsafe().node_text_content();
        if node_text_content.is_empty() {
            // The addition of zero-width space here forces the text input to have an inline formatting
            // context that might otherwise be trimmed if there's no text. This is important to ensure
            // that the input element is at least as tall as the line gap of the caret:
            // <https://drafts.csswg.org/css-ui/#element-with-default-preferred-size>.
            //
            // This is also used to ensure that the caret will still be rendered when the input is empty.
            // TODO: Is there a less hacky way to do this?
            handler.handle_text(parent_element_info, "\u{200B}".into());
        } else {
            handler.handle_text(parent_element_info, node_text_content);
        }
    } else {
        for child in iter_child_nodes(parent_element_info.node) {
            if child.is_text_node() {
                let info = NodeAndStyleInfo::new(
                    child,
                    child.style(&context.style_context),
                    child.take_restyle_damage(),
                );
                handler.handle_text(&info, child.to_threadsafe().node_text_content());
            } else if child.is_element() {
                traverse_element(child, context, handler);
            }
        }
    }

    traverse_eager_pseudo_element(PseudoElement::After, parent_element_info, context, handler);
}

fn traverse_element<'dom>(
    element: ServoLayoutNode<'dom>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom>,
) {
    element.unset_all_pseudo_boxes();

    let replaced = ReplacedContents::for_element(element, context);
    let style = element.style(&context.style_context);
    let damage = element.take_restyle_damage();
    let info = NodeAndStyleInfo::new(element, style, damage);

    match Display::from(info.style.get_box().display) {
        Display::None => element.unset_all_boxes(),
        Display::Contents => {
            if replaced.is_some() {
                // `display: content` on a replaced element computes to `display: none`
                // <https://drafts.csswg.org/css-display-3/#valdef-display-contents>
                element.unset_all_boxes()
            } else {
                let shared_inline_styles: SharedInlineStyles = (&info).into();
                element
                    .element_box_slot()
                    .set(LayoutBox::DisplayContents(shared_inline_styles.clone()));

                handler.enter_display_contents(shared_inline_styles);
                traverse_children_of(&info, context, handler);
                handler.leave_display_contents();
            }
        },
        Display::GeneratingBox(display) => {
            let contents = if let Some(replaced) = replaced {
                Contents::Replaced(replaced)
            } else if matches!(
                element.type_id(),
                LayoutNodeType::Element(
                    LayoutElementType::HTMLInputElement | LayoutElementType::HTMLTextAreaElement
                )
            ) {
                NonReplacedContents::OfTextControl.into()
            } else {
                NonReplacedContents::OfElement.into()
            };
            let display = display.used_value_for_contents(&contents);
            let box_slot = element.element_box_slot();
            handler.handle_element(&info, display, contents, box_slot);
        },
    }
}

fn traverse_eager_pseudo_element<'dom>(
    pseudo_element_type: PseudoElement,
    node_info: &NodeAndStyleInfo<'dom>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom>,
) {
    assert!(pseudo_element_type.is_eager());

    // If this node doesn't have this eager pseudo-element, exit early. This depends on
    // the style applied to the element.
    let Some(pseudo_element_info) = node_info.pseudo(context, pseudo_element_type) else {
        return;
    };
    if pseudo_element_info.style.ineffective_content_property() {
        return;
    }

    match Display::from(pseudo_element_info.style.get_box().display) {
        Display::None => {},
        Display::Contents => {
            let items = generate_pseudo_element_content(&pseudo_element_info, context);
            let box_slot = pseudo_element_info
                .node
                .pseudo_element_box_slot(pseudo_element_type);
            let shared_inline_styles: SharedInlineStyles = (&pseudo_element_info).into();
            box_slot.set(LayoutBox::DisplayContents(shared_inline_styles.clone()));

            handler.enter_display_contents(shared_inline_styles);
            traverse_pseudo_element_contents(&pseudo_element_info, context, handler, items);
            handler.leave_display_contents();
        },
        Display::GeneratingBox(display) => {
            let items = generate_pseudo_element_content(&pseudo_element_info, context);
            let box_slot = pseudo_element_info
                .node
                .pseudo_element_box_slot(pseudo_element_type);
            let contents = NonReplacedContents::OfPseudoElement(items).into();
            handler.handle_element(&pseudo_element_info, display, contents, box_slot);
        },
    }
}

fn traverse_pseudo_element_contents<'dom>(
    info: &NodeAndStyleInfo<'dom>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom>,
    items: Vec<PseudoElementContentItem>,
) {
    let mut anonymous_info = None;
    for item in items {
        match item {
            PseudoElementContentItem::Text(text) => handler.handle_text(info, text.into()),
            PseudoElementContentItem::Replaced(contents) => {
                let anonymous_info = anonymous_info.get_or_insert_with(|| {
                    info.pseudo(context, PseudoElement::ServoAnonymousBox)
                        .unwrap_or_else(|| info.clone())
                });
                let display_inline = DisplayGeneratingBox::OutsideInside {
                    outside: DisplayOutside::Inline,
                    inside: DisplayInside::Flow {
                        is_list_item: false,
                    },
                };
                // `display` is not inherited, so we get the initial value
                debug_assert!(
                    Display::from(anonymous_info.style.get_box().display) ==
                        Display::GeneratingBox(display_inline)
                );
                handler.handle_element(
                    anonymous_info,
                    display_inline,
                    Contents::Replaced(contents),
                    info.node
                        .pseudo_element_box_slot(PseudoElement::ServoAnonymousBox),
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

impl NonReplacedContents {
    pub(crate) fn traverse<'dom>(
        self,
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'dom>,
        handler: &mut impl TraversalHandler<'dom>,
    ) {
        match self {
            NonReplacedContents::OfElement | NonReplacedContents::OfTextControl => {
                traverse_children_of(info, context, handler)
            },
            NonReplacedContents::OfPseudoElement(items) => {
                traverse_pseudo_element_contents(info, context, handler, items)
            },
        }
    }
}

fn get_quote_from_pair<I, S>(item: &ContentItem<I>, opening: &S, closing: &S) -> String
where
    S: ToString + ?Sized,
{
    match item {
        ContentItem::OpenQuote => opening.to_string(),
        ContentItem::CloseQuote => closing.to_string(),
        _ => unreachable!("Got an unexpected ContentItem type when processing quotes."),
    }
}

/// <https://www.w3.org/TR/CSS2/generate.html#propdef-content>
fn generate_pseudo_element_content(
    pseudo_element_info: &NodeAndStyleInfo,
    context: &LayoutContext,
) -> Vec<PseudoElementContentItem> {
    match &pseudo_element_info.style.get_counters().content {
        Content::Items(items) => {
            let mut vec = vec![];
            for item in items.items.iter() {
                match item {
                    ContentItem::String(s) => {
                        vec.push(PseudoElementContentItem::Text(s.to_string()));
                    },
                    ContentItem::Attr(attr) => {
                        let element = pseudo_element_info
                            .node
                            .to_threadsafe()
                            .as_element()
                            .expect("Expected an element");

                        // From
                        // <https://html.spec.whatwg.org/multipage/#case-sensitivity-of-the-css-%27attr%28%29%27-function>
                        //
                        // > CSS Values and Units leaves the case-sensitivity of attribute names for
                        // > the purpose of the `attr()` function to be defined by the host language.
                        // > [[CSSVALUES]].
                        // >
                        // > When comparing the attribute name part of a CSS `attr()`function to the
                        // > names of namespace-less attributes on HTML elements in HTML documents,
                        // > the name part of the CSS `attr()` function must first be converted to
                        // > ASCII lowercase. The same function when compared to other attributes must
                        // > be compared according to its original case. In both cases, to match the
                        // > values must be identical to each other (and therefore the comparison is
                        // > case sensitive).
                        let attr_name = match element.is_html_element_in_html_document() {
                            true => &*attr.attribute.to_ascii_lowercase(),
                            false => &*attr.attribute,
                        };

                        let attr_val =
                            element.get_attr(&attr.namespace_url, &LocalName::from(attr_name));
                        vec.push(PseudoElementContentItem::Text(
                            attr_val.map_or("".to_string(), |s| s.to_string()),
                        ));
                    },
                    ContentItem::Image(image) => {
                        if let Some(replaced_content) =
                            ReplacedContents::from_image(pseudo_element_info.node, context, image)
                        {
                            vec.push(PseudoElementContentItem::Replaced(replaced_content));
                        }
                    },
                    ContentItem::OpenQuote | ContentItem::CloseQuote => {
                        // TODO(xiaochengh): calculate quote depth
                        let maybe_quote = match &pseudo_element_info.style.get_list().quotes {
                            Quotes::QuoteList(quote_list) => {
                                quote_list.0.first().map(|quote_pair| {
                                    get_quote_from_pair(
                                        item,
                                        &*quote_pair.opening,
                                        &*quote_pair.closing,
                                    )
                                })
                            },
                            Quotes::Auto => {
                                let lang = &pseudo_element_info.style.get_font()._x_lang;
                                let quotes = quotes_for_lang(lang.0.as_ref(), 0);
                                Some(get_quote_from_pair(item, &quotes.opening, &quotes.closing))
                            },
                        };
                        if let Some(quote) = maybe_quote {
                            vec.push(PseudoElementContentItem::Text(quote));
                        }
                    },
                    ContentItem::Counter(_, _) |
                    ContentItem::Counters(_, _, _) |
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

pub enum ChildNodeIterator<'dom> {
    /// Iterating over the children of a node
    Node(Option<ServoLayoutNode<'dom>>),
    /// Iterating over the assigned nodes of a `HTMLSlotElement`
    Slottables(<Vec<ServoLayoutNode<'dom>> as IntoIterator>::IntoIter),
}

pub(crate) fn iter_child_nodes(parent: ServoLayoutNode<'_>) -> ChildNodeIterator<'_> {
    if let Some(element) = parent.as_element() {
        if let Some(shadow) = element.shadow_root() {
            return iter_child_nodes(shadow.as_node());
        };

        let slotted_nodes = element.slotted_nodes();
        if !slotted_nodes.is_empty() {
            #[allow(clippy::unnecessary_to_owned)] // Clippy is wrong.
            return ChildNodeIterator::Slottables(slotted_nodes.to_owned().into_iter());
        }
    }

    let first = parent.first_child();
    ChildNodeIterator::Node(first)
}

impl<'dom> Iterator for ChildNodeIterator<'dom> {
    type Item = ServoLayoutNode<'dom>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Node(node) => {
                let old = *node;
                *node = old?.next_sibling();
                old
            },
            Self::Slottables(slots) => slots.next(),
        }
    }
}

impl FusedIterator for ChildNodeIterator<'_> {}
