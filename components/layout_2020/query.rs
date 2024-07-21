/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by layout.
use std::sync::Arc;

use app_units::Au;
use euclid::default::{Point2D, Rect};
use euclid::{SideOffsets2D, Size2D, Vector2D};
use log::warn;
use script_layout_interface::wrapper_traits::{
    LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_layout_interface::OffsetParentResponse;
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use style::computed_values::position::T as Position;
use style::context::{QuirksMode, SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use style::dom::{OpaqueNode, TElement};
use style::properties::style_structs::Font;
use style::properties::{
    parse_one_declaration_into, ComputedValues, Importance, LonghandId, PropertyDeclarationBlock,
    PropertyDeclarationId, PropertyId, ShorthandId, SourcePropertyDeclaration,
};
use style::selector_parser::PseudoElement;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style::stylist::RuleInclusion;
use style::traversal::resolve_style;
use style::values::generics::font::LineHeight;
use style_traits::{ParsingMode, ToCss};

use crate::fragment_tree::{BoxFragment, Fragment, FragmentFlags, FragmentTree, Tag};

pub fn process_content_box_request(
    requested_node: OpaqueNode,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> Option<Rect<Au>> {
    let rects = fragment_tree?.get_content_boxes_for_node(requested_node);
    if rects.is_empty() {
        return None;
    }

    Some(
        rects
            .iter()
            .fold(Rect::zero(), |unioned_rect, rect| rect.union(&unioned_rect)),
    )
}

pub fn process_content_boxes_request(
    requested_node: OpaqueNode,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> Vec<Rect<Au>> {
    fragment_tree
        .map(|tree| tree.get_content_boxes_for_node(requested_node))
        .unwrap_or_default()
}

pub fn process_node_geometry_request(
    requested_node: OpaqueNode,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> Rect<i32> {
    if let Some(fragment_tree) = fragment_tree {
        fragment_tree.get_border_dimensions_for_node(requested_node)
    } else {
        Rect::zero()
    }
}

/// <https://drafts.csswg.org/cssom-view/#scrolling-area>
pub fn process_node_scroll_area_request(
    requested_node: Option<OpaqueNode>,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> Rect<i32> {
    let rect = match (fragment_tree, requested_node) {
        (Some(tree), Some(node)) => tree.get_scrolling_area_for_node(node),
        (Some(tree), None) => tree.get_scrolling_area_for_viewport(),
        _ => return Rect::zero(),
    };

    Rect::new(
        Point2D::new(rect.origin.x.to_f32_px(), rect.origin.y.to_f32_px()),
        Size2D::new(rect.size.width.to_f32_px(), rect.size.height.to_f32_px()),
    )
    .round()
    .to_i32()
    .to_untyped()
}

/// Return the resolved value of property for a given (pseudo)element.
/// <https://drafts.csswg.org/cssom/#resolved-value>
pub fn process_resolved_style_request<'dom>(
    context: &SharedStyleContext,
    node: impl LayoutNode<'dom>,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> String {
    if !node.as_element().unwrap().has_data() {
        return process_resolved_style_request_for_unstyled_node(context, node, pseudo, property);
    }

    // We call process_resolved_style_request after performing a whole-document
    // traversal, so in the common case, the element is styled.
    let layout_element = node.to_threadsafe().as_element().unwrap();
    let layout_element = match *pseudo {
        None => Some(layout_element),
        Some(PseudoElement::Before) => layout_element.get_before_pseudo(),
        Some(PseudoElement::After) => layout_element.get_after_pseudo(),
        Some(_) => {
            warn!("Got unexpected pseudo element type!");
            None
        },
    };

    let layout_element = match layout_element {
        None => {
            // The pseudo doesn't exist, return nothing.  Chrome seems to query
            // the element itself in this case, Firefox uses the resolved value.
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=29006
            return String::new();
        },
        Some(layout_element) => layout_element,
    };

    let style = &*layout_element.resolved_style();
    let longhand_id = match *property {
        PropertyId::NonCustom(id) => match id.longhand_or_shorthand() {
            Ok(longhand_id) => longhand_id,
            Err(shorthand_id) => return shorthand_to_css_string(shorthand_id, style),
        },
        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name));
        },
    }
    .to_physical(style.writing_mode);

    let computed_style =
        || style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id));

    let tag_to_find = Tag::new_pseudo(node.opaque(), *pseudo);

    // https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle
    // Here we are trying to conform to the specification that says that getComputedStyle
    // should return the used values in certain circumstances. For size and positional
    // properties we might need to walk the Fragment tree to figure those out. We always
    // fall back to returning the computed value.

    // For line height, the resolved value is the computed value if it
    // is "normal" and the used value otherwise.
    if longhand_id == LonghandId::LineHeight {
        let font = style.get_font();
        let font_size = font.font_size.computed_size();
        return match font.line_height {
            LineHeight::Normal => computed_style(),
            LineHeight::Number(value) => (font_size * value.0).to_css_string(),
            LineHeight::Length(value) => value.0.to_css_string(),
        };
    }

    // https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle
    // The properties that we calculate below all resolve to the computed value
    // when the element is display:none or display:contents.
    let display = style.get_box().display;
    if display.is_none() || display.is_contents() {
        return computed_style();
    }

    let fragment_tree = match fragment_tree {
        Some(fragment_tree) => fragment_tree,
        None => return computed_style(),
    };
    fragment_tree
        .find(|fragment, _, containing_block| {
            if Some(tag_to_find) != fragment.tag() {
                return None;
            }

            let (content_rect, margins, padding) = match fragment {
                Fragment::Box(ref box_fragment) | Fragment::Float(ref box_fragment) => {
                    if style.get_box().position != Position::Static {
                        let resolved_insets = || {
                            box_fragment.calculate_resolved_insets_if_positioned(containing_block)
                        };
                        match longhand_id {
                            LonghandId::Top => return Some(resolved_insets().top.to_css_string()),
                            LonghandId::Right => {
                                return Some(resolved_insets().right.to_css_string())
                            },
                            LonghandId::Bottom => {
                                return Some(resolved_insets().bottom.to_css_string())
                            },
                            LonghandId::Left => {
                                return Some(resolved_insets().left.to_css_string())
                            },
                            _ => {},
                        }
                    }
                    let content_rect = box_fragment
                        .content_rect
                        .to_physical(box_fragment.style.writing_mode, containing_block);
                    let margins = box_fragment
                        .margin
                        .to_physical(box_fragment.style.writing_mode);
                    let padding = box_fragment
                        .padding
                        .to_physical(box_fragment.style.writing_mode);
                    (content_rect, margins, padding)
                },
                Fragment::Positioning(positioning_fragment) => {
                    let content_rect = positioning_fragment
                        .rect
                        .to_physical(positioning_fragment.writing_mode, containing_block);
                    (content_rect, SideOffsets2D::zero(), SideOffsets2D::zero())
                },
                _ => return None,
            };

            // https://drafts.csswg.org/cssom/#resolved-value-special-case-property-like-height
            // > If the property applies to the element or pseudo-element and the resolved value of the
            // > display property is not none or contents, then the resolved value is the used value.
            // > Otherwise the resolved value is the computed value.
            //
            // However, all browsers ignore that for margin and padding properties, and resolve to a length
            // even if the property doesn't apply: https://github.com/w3c/csswg-drafts/issues/10391
            match longhand_id {
                LonghandId::Width if resolved_size_should_be_used_value(fragment) => {
                    Some(content_rect.size.width)
                },
                LonghandId::Height if resolved_size_should_be_used_value(fragment) => {
                    Some(content_rect.size.height)
                },
                LonghandId::MarginBottom => Some(margins.bottom),
                LonghandId::MarginTop => Some(margins.top),
                LonghandId::MarginLeft => Some(margins.left),
                LonghandId::MarginRight => Some(margins.right),
                LonghandId::PaddingBottom => Some(padding.bottom),
                LonghandId::PaddingTop => Some(padding.top),
                LonghandId::PaddingLeft => Some(padding.left),
                LonghandId::PaddingRight => Some(padding.right),
                _ => None,
            }
            .map(|value| value.to_css_string())
        })
        .unwrap_or_else(computed_style)
}

fn resolved_size_should_be_used_value(fragment: &Fragment) -> bool {
    // https://drafts.csswg.org/css-sizing-3/#preferred-size-properties
    // > Applies to: all elements except non-replaced inlines
    match fragment {
        Fragment::Box(box_fragment) => {
            !box_fragment.style.get_box().display.is_inline_flow() ||
                fragment.base().map_or(false, |base| {
                    base.flags.contains(FragmentFlags::IS_REPLACED)
                })
        },
        Fragment::Float(_) |
        Fragment::Positioning(_) |
        Fragment::AbsoluteOrFixedPositioned(_) |
        Fragment::Image(_) |
        Fragment::IFrame(_) => true,
        Fragment::Text(_) => false,
    }
}

pub fn process_resolved_style_request_for_unstyled_node<'dom>(
    context: &SharedStyleContext,
    node: impl LayoutNode<'dom>,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
) -> String {
    // In a display: none subtree. No pseudo-element exists.
    if pseudo.is_some() {
        return String::new();
    }

    let mut tlc = ThreadLocalStyleContext::new();
    let mut context = StyleContext {
        shared: context,
        thread_local: &mut tlc,
    };

    let element = node.as_element().unwrap();
    let styles = resolve_style(
        &mut context,
        element,
        RuleInclusion::All,
        pseudo.as_ref(),
        None,
    );
    let style = styles.primary();
    let longhand_id = match *property {
        PropertyId::NonCustom(id) => match id.longhand_or_shorthand() {
            Ok(longhand_id) => longhand_id,
            Err(shorthand_id) => return shorthand_to_css_string(shorthand_id, style),
        },
        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name));
        },
    };

    // No need to care about used values here, since we're on a display: none
    // subtree, use the resolved value.
    style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id))
}

fn shorthand_to_css_string(
    id: style::properties::ShorthandId,
    style: &style::properties::ComputedValues,
) -> String {
    use style::values::resolved::Context;
    let mut block = PropertyDeclarationBlock::new();
    let mut dest = String::new();
    for longhand in id.longhands() {
        block.push(
            style.computed_or_resolved_declaration(longhand, Some(&Context { style })),
            Importance::Normal,
        );
    }
    match block.shorthand_to_css(id, &mut dest) {
        Ok(_) => dest.to_owned(),
        Err(_) => String::new(),
    }
}

pub fn process_offset_parent_query(
    node: OpaqueNode,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> OffsetParentResponse {
    process_offset_parent_query_inner(node, fragment_tree).unwrap_or_default()
}

#[inline]
fn process_offset_parent_query_inner(
    node: OpaqueNode,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> Option<OffsetParentResponse> {
    let fragment_tree = fragment_tree?;

    struct NodeOffsetBoxInfo {
        border_box: Rect<Au>,
        offset_parent_node_address: Option<OpaqueNode>,
    }

    // https://www.w3.org/TR/2016/WD-cssom-view-1-20160317/#extensions-to-the-htmlelement-interface
    let mut parent_node_addresses = Vec::new();
    let tag_to_find = Tag::new(node);
    let node_offset_box = fragment_tree.find(|fragment, level, containing_block| {
        let base = fragment.base()?;
        let is_body_element = base
            .flags
            .contains(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT);

        if fragment.tag() == Some(tag_to_find) {
            // Only consider the first fragment of the node found as per a
            // possible interpretation of the specification: "[...] return the
            // y-coordinate of the top border edge of the first CSS layout box
            // associated with the element [...]"
            //
            // FIXME: Browsers implement this all differently (e.g., [1]) -
            // Firefox does returns the union of all layout elements of some
            // sort. Chrome returns the first fragment for a block element (the
            // same as ours) or the union of all associated fragments in the
            // first containing block fragment for an inline element. We could
            // implement Chrome's behavior, but our fragment tree currently
            // provides insufficient information.
            //
            // [1]: https://github.com/w3c/csswg-drafts/issues/4541
            let fragment_relative_rect = match fragment {
                Fragment::Box(fragment) | Fragment::Float(fragment) => fragment
                    .border_rect()
                    .to_physical(fragment.style.writing_mode, containing_block),
                Fragment::Text(fragment) => fragment
                    .rect
                    .to_physical(fragment.parent_style.writing_mode, containing_block),
                Fragment::Positioning(fragment) => fragment
                    .rect
                    .to_physical(fragment.writing_mode, containing_block),
                Fragment::AbsoluteOrFixedPositioned(_) |
                Fragment::Image(_) |
                Fragment::IFrame(_) => unreachable!(),
            };

            let mut border_box = fragment_relative_rect.translate(containing_block.origin.to_vector()).to_untyped();

            // "If any of the following holds true return null and terminate
            // this algorithm: [...] The element’s computed value of the
            // `position` property is `fixed`."
            let is_fixed = matches!(
                fragment, Fragment::Box(fragment) if fragment.style.get_box().position == Position::Fixed
            );

            if is_body_element {
                // "If the element is the HTML body element or [...] return zero
                // and terminate this algorithm."
                border_box.origin = Point2D::zero();
            }

            let offset_parent_node_address = if is_fixed {
                None
            } else {
                // Find the nearest ancestor element eligible as `offsetParent`.
                parent_node_addresses[..level]
                    .iter()
                    .rev()
                    .cloned()
                    .find_map(std::convert::identity)
            };

            Some(NodeOffsetBoxInfo {
                border_box,
                offset_parent_node_address,
            })
        } else {
            // Record the paths of the nodes being traversed.
            let parent_node_address = match fragment {
                Fragment::Box(fragment) | Fragment::Float(fragment) => {
                    let is_eligible_parent = is_eligible_parent(fragment);
                    match base.tag {
                        Some(tag) if is_eligible_parent && !tag.is_pseudo() => Some(tag.node),
                        _ => None,
                    }
                },
                Fragment::AbsoluteOrFixedPositioned(_) |
                Fragment::IFrame(_) |
                Fragment::Image(_) |
                Fragment::Positioning(_) |
                Fragment::Text(_) => None,
            };

            while parent_node_addresses.len() <= level {
                parent_node_addresses.push(None);
            }
            parent_node_addresses[level] = parent_node_address;
            None
        }
    });

    // Bail out if the element doesn't have an associated fragment.
    // "If any of the following holds true return null and terminate this
    // algorithm: [...] The element does not have an associated CSS layout box."
    // (`offsetParent`) "If the element is the HTML body element [...] return
    // zero and terminate this algorithm." (others)
    let node_offset_box = node_offset_box?;

    let offset_parent_padding_box_corner = node_offset_box
        .offset_parent_node_address
        .map(|offset_parent_node_address| {
            // Find the top and left padding edges of "the first CSS layout box
            // associated with the `offsetParent` of the element".
            //
            // Since we saw `offset_parent_node_address` once, we should be able
            // to find it again.
            let offset_parent_node_tag = Tag::new(offset_parent_node_address);
            fragment_tree
                .find(|fragment, _, containing_block| {
                    match fragment {
                        Fragment::Box(fragment) | Fragment::Float(fragment) => {
                            if fragment.base.tag == Some(offset_parent_node_tag) {
                                // Again, take the *first* associated CSS layout box.
                                let padding_box_corner = fragment
                                    .padding_rect()
                                    .to_physical(fragment.style.writing_mode, containing_block)
                                    .origin
                                    .to_vector() +
                                    containing_block.origin.to_vector();
                                let padding_box_corner = padding_box_corner.to_untyped();
                                Some(padding_box_corner)
                            } else {
                                None
                            }
                        },
                        Fragment::AbsoluteOrFixedPositioned(_) |
                        Fragment::Text(_) |
                        Fragment::Image(_) |
                        Fragment::IFrame(_) |
                        Fragment::Positioning(_) => None,
                    }
                })
                .unwrap()
        })
        // "If the offsetParent of the element is null," subtract zero in the
        // following step.
        .unwrap_or(Vector2D::zero());

    Some(OffsetParentResponse {
        node_address: node_offset_box.offset_parent_node_address.map(Into::into),
        // "Return the result of subtracting the x-coordinate of the left
        // padding edge of the first CSS layout box associated with the
        // `offsetParent` of the element from the x-coordinate of the left
        // border edge of the first CSS layout box associated with the element,
        // relative to the initial containing block origin, ignoring any
        // transforms that apply to the element and its ancestors." (and vice
        // versa for the top border edge)
        rect: node_offset_box
            .border_box
            .translate(-offset_parent_padding_box_corner),
    })
}

/// Returns whether or not the element with the given style and body element determination
/// is eligible to be a parent element for offset* queries.
///
/// From <https://www.w3.org/TR/cssom-view-1/#dom-htmlelement-offsetparent>:
/// >
/// > Return the nearest ancestor element of the element for which at least one of the following is
/// > true and terminate this algorithm if such an ancestor is found:
/// >   1. The computed value of the position property is not static.
/// >   2. It is the HTML body element.
/// >   3. The computed value of the position property of the element is static and the ancestor is
/// >      one of the following HTML elements: td, th, or table.
fn is_eligible_parent(fragment: &BoxFragment) -> bool {
    fragment
        .base
        .flags
        .contains(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT) ||
        fragment.style.get_box().position != Position::Static ||
        fragment
            .base
            .flags
            .contains(FragmentFlags::IS_TABLE_TH_OR_TD_ELEMENT)
}

// https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute
pub fn process_element_inner_text_query<'dom>(_node: impl LayoutNode<'dom>) -> String {
    "".to_owned()
}

pub fn process_text_index_request(_node: OpaqueNode, _point: Point2D<Au>) -> Option<usize> {
    None
}

pub fn process_resolved_font_style_query<'dom, E>(
    context: &SharedStyleContext,
    node: E,
    value: &str,
    url_data: ServoUrl,
    shared_lock: &SharedRwLock,
) -> Option<ServoArc<Font>>
where
    E: LayoutNode<'dom>,
{
    fn create_font_declaration(
        value: &str,
        url_data: &ServoUrl,
        quirks_mode: QuirksMode,
    ) -> Option<PropertyDeclarationBlock> {
        let mut declarations = SourcePropertyDeclaration::default();
        let result = parse_one_declaration_into(
            &mut declarations,
            PropertyId::NonCustom(ShorthandId::Font.into()),
            value,
            Origin::Author,
            &UrlExtraData(url_data.get_arc()),
            None,
            ParsingMode::DEFAULT,
            quirks_mode,
            CssRuleType::Style,
        );
        let declarations = match result {
            Ok(()) => {
                let mut block = PropertyDeclarationBlock::new();
                block.extend(declarations.drain(), Importance::Normal);
                block
            },
            Err(_) => return None,
        };
        // TODO: Force to set line-height property to 'normal' font property.
        Some(declarations)
    }
    fn resolve_for_declarations<'dom, E>(
        context: &SharedStyleContext,
        parent_style: Option<&ComputedValues>,
        declarations: PropertyDeclarationBlock,
        shared_lock: &SharedRwLock,
    ) -> ServoArc<ComputedValues>
    where
        E: LayoutNode<'dom>,
    {
        let parent_style = match parent_style {
            Some(parent) => parent,
            None => context.stylist.device().default_computed_values(),
        };
        context
            .stylist
            .compute_for_declarations::<E::ConcreteElement>(
                &context.guards,
                parent_style,
                ServoArc::new(shared_lock.wrap(declarations)),
            )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    // 1. Parse the given font property value
    let quirks_mode = context.quirks_mode();
    let declarations = create_font_declaration(value, &url_data, quirks_mode)?;

    // TODO: Reject 'inherit' and 'initial' values for the font property.

    // 2. Get resolved styles for the parent element
    let element = node.as_element().unwrap();
    let parent_style = if node.is_connected() {
        if element.has_data() {
            node.to_threadsafe().as_element().unwrap().resolved_style()
        } else {
            let mut tlc = ThreadLocalStyleContext::new();
            let mut context = StyleContext {
                shared: context,
                thread_local: &mut tlc,
            };
            let styles = resolve_style(&mut context, element, RuleInclusion::All, None, None);
            styles.primary().clone()
        }
    } else {
        let default_declarations =
            create_font_declaration("10px sans-serif", &url_data, quirks_mode).unwrap();
        resolve_for_declarations::<E>(context, None, default_declarations, shared_lock)
    };

    // 3. Resolve the parsed value with resolved styles of the parent element
    let computed_values =
        resolve_for_declarations::<E>(context, Some(&*parent_style), declarations, shared_lock);

    Some(computed_values.clone_font())
}
