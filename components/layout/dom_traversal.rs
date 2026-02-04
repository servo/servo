/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use html5ever::LocalName;
use layout_api::wrapper_traits::{
    PseudoElementChain, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use layout_api::{LayoutDamage, LayoutElementType, LayoutNodeType};
use script::layout_dom::ServoThreadSafeLayoutNode;
use selectors::Element as SelectorsElement;
use servo_arc::Arc as ServoArc;
use style::dom::NodeInfo;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style::values::generics::counters::{Content, ContentItem};
use style::values::specified::Quotes;

use crate::context::LayoutContext;
use crate::dom::{BoxSlot, LayoutBox, NodeExt};
use crate::flow::inline::SharedInlineStyles;
use crate::quotes::quotes_for_lang;
use crate::replaced::ReplacedContents;
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside, DisplayOutside};

/// A data structure used to pass and store related layout information together to
/// avoid having to repeat the same arguments in argument lists.
#[derive(Clone)]
pub(crate) struct NodeAndStyleInfo<'dom> {
    pub node: ServoThreadSafeLayoutNode<'dom>,
    pub style: ServoArc<ComputedValues>,
    pub damage: LayoutDamage,
}

impl<'dom> NodeAndStyleInfo<'dom> {
    pub(crate) fn new(
        node: ServoThreadSafeLayoutNode<'dom>,
        style: ServoArc<ComputedValues>,
        damage: LayoutDamage,
    ) -> Self {
        Self {
            node,
            style,
            damage,
        }
    }

    pub(crate) fn pseudo_element_chain(&self) -> PseudoElementChain {
        self.node.pseudo_element_chain()
    }

    pub(crate) fn with_pseudo_element(
        &self,
        context: &LayoutContext,
        pseudo_element_type: PseudoElement,
    ) -> Option<Self> {
        let element = self.node.as_element()?.with_pseudo(pseudo_element_type)?;
        let style = element.style(&context.style_context);
        Some(NodeAndStyleInfo {
            node: element.as_node(),
            style,
            damage: self.damage,
        })
    }
}

#[derive(Debug)]
pub(super) enum Contents {
    /// Any kind of content that is not replaced nor a widget, including the contents of pseudo-elements.
    NonReplaced(NonReplacedContents),
    /// A widget with native appearance. This has several behavior in common with replaced elements,
    /// but isn't fully replaced (see discussion in <https://github.com/w3c/csswg-drafts/issues/12876>).
    /// Examples: `<input>`, `<textarea>`, `<select>`...
    /// <https://drafts.csswg.org/css-ui/#widget>
    Widget(NonReplacedContents),
    /// Example: an `<img src=…>` element.
    /// <https://drafts.csswg.org/css2/conform.html#replaced-element>
    Replaced(ReplacedContents),
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
    parent_element_info
        .node
        .set_uses_content_attribute_with_attr(false);

    let is_element = parent_element_info.pseudo_element_chain().is_empty();
    if is_element {
        traverse_eager_pseudo_element(PseudoElement::Before, parent_element_info, context, handler);
    }

    for child in parent_element_info.node.children() {
        if child.is_text_node() {
            let info = NodeAndStyleInfo::new(
                child,
                child.style(&context.style_context),
                child.take_restyle_damage(),
            );
            handler.handle_text(&info, child.text_content());
        } else if child.is_element() {
            traverse_element(child, context, handler);
        }
    }

    if is_element {
        traverse_eager_pseudo_element(PseudoElement::After, parent_element_info, context, handler);

        traverse_picker_icon_pseudo_element(parent_element_info, context, handler);
    }
}

fn traverse_element<'dom>(
    element: ServoThreadSafeLayoutNode<'dom>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom>,
) {
    let damage = element.take_restyle_damage();
    if damage.has_box_damage() {
        element.unset_all_pseudo_boxes();
    }

    let style = element.style(&context.style_context);
    let info = NodeAndStyleInfo::new(element, style, damage);

    match Display::from(info.style.get_box().display) {
        Display::None => element.unset_all_boxes(),
        Display::Contents => {
            if ReplacedContents::for_element(element, context).is_some() {
                // `display: content` on a replaced element computes to `display: none`
                // <https://drafts.csswg.org/css-display-3/#valdef-display-contents>
                element.unset_all_boxes()
            } else {
                let shared_inline_styles: SharedInlineStyles = (&info).into();
                element
                    .box_slot()
                    .set(LayoutBox::DisplayContents(shared_inline_styles.clone()));

                handler.enter_display_contents(shared_inline_styles);
                traverse_children_of(&info, context, handler);
                handler.leave_display_contents();
            }
        },
        Display::GeneratingBox(display) => {
            let contents = Contents::for_element(element, context);
            let display = display.used_value_for_contents(&contents);
            let box_slot = element.box_slot();
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
    let Some(pseudo_element_info) = node_info.with_pseudo_element(context, pseudo_element_type)
    else {
        return;
    };
    if pseudo_element_info.style.ineffective_content_property() {
        return;
    }

    match Display::from(pseudo_element_info.style.get_box().display) {
        Display::None => {},
        Display::Contents => {
            let items = generate_pseudo_element_content(&pseudo_element_info, context);
            let box_slot = pseudo_element_info.node.box_slot();
            let shared_inline_styles: SharedInlineStyles = (&pseudo_element_info).into();
            box_slot.set(LayoutBox::DisplayContents(shared_inline_styles.clone()));

            handler.enter_display_contents(shared_inline_styles);
            traverse_pseudo_element_contents(&pseudo_element_info, context, handler, items);
            handler.leave_display_contents();
        },
        Display::GeneratingBox(display) => {
            let items = generate_pseudo_element_content(&pseudo_element_info, context);
            let box_slot = pseudo_element_info.node.box_slot();
            let contents = Contents::for_pseudo_element(items);
            handler.handle_element(&pseudo_element_info, display, contents, box_slot);
        },
    }
}

fn traverse_picker_icon_pseudo_element<'dom>(
    info: &NodeAndStyleInfo<'dom>,
    context: &LayoutContext,
    handler: &mut impl TraversalHandler<'dom>,
) {
    let Some(element) = info.node.as_element() else {
        return;
    };

    if !matches!(
        element.type_id(),
        Some(LayoutNodeType::Element(
            LayoutElementType::HTMLSelectElement
        ))
    ) {
        return;
    }

    let Some(pseudo_element_info) =
        info.with_pseudo_element(context, PseudoElement::ServoPickerIcon)
    else {
        return;
    };

    if pseudo_element_info.style.ineffective_content_property() {
        return;
    }

    match Display::from(pseudo_element_info.style.get_box().display) {
        Display::None => {},
        Display::Contents => {
            let items = generate_pseudo_element_content(&pseudo_element_info, context);
            let box_slot = pseudo_element_info.node.box_slot();
            let shared_inline_styles: SharedInlineStyles = (&pseudo_element_info).into();
            box_slot.set(LayoutBox::DisplayContents(shared_inline_styles.clone()));

            handler.enter_display_contents(shared_inline_styles);
            traverse_pseudo_element_contents(&pseudo_element_info, context, handler, items);
            handler.leave_display_contents();
        },
        Display::GeneratingBox(display) => {
            let items = generate_pseudo_element_content(&pseudo_element_info, context);
            let box_slot = pseudo_element_info.node.box_slot();
            let contents = Contents::for_pseudo_element(items);
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
                    info.with_pseudo_element(context, PseudoElement::ServoAnonymousBox)
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
                    anonymous_info.node.box_slot(),
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

    pub(crate) fn for_element(
        node: ServoThreadSafeLayoutNode<'_>,
        context: &LayoutContext,
    ) -> Self {
        if let Some(replaced) = ReplacedContents::for_element(node, context) {
            return Self::Replaced(replaced);
        }
        let is_widget = matches!(
            node.type_id(),
            Some(LayoutNodeType::Element(
                LayoutElementType::HTMLInputElement |
                    LayoutElementType::HTMLSelectElement |
                    LayoutElementType::HTMLTextAreaElement
            ))
        );
        if is_widget {
            Self::Widget(NonReplacedContents::OfElement)
        } else {
            Self::NonReplaced(NonReplacedContents::OfElement)
        }
    }

    pub(crate) fn for_pseudo_element(contents: Vec<PseudoElementContentItem>) -> Self {
        Self::NonReplaced(NonReplacedContents::OfPseudoElement(contents))
    }

    pub(crate) fn non_replaced_contents(self) -> Option<NonReplacedContents> {
        match self {
            Self::NonReplaced(contents) | Self::Widget(contents) => Some(contents),
            Self::Replaced(_) => None,
        }
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
            NonReplacedContents::OfElement => traverse_children_of(info, context, handler),
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

                        pseudo_element_info
                            .node
                            .set_uses_content_attribute_with_attr(true);
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
