/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by layout.
use std::rc::Rc;

use app_units::Au;
use euclid::default::{Point2D, Rect};
use euclid::{SideOffsets2D, Size2D};
use itertools::Itertools;
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use layout_api::{LayoutElementType, LayoutNodeType, OffsetParentResponse};
use script::layout_dom::ServoLayoutNode;
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use style::computed_values::display::T as Display;
use style::computed_values::position::T as Position;
use style::computed_values::visibility::T as Visibility;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapseValue;
use style::context::{QuirksMode, SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use style::dom::{NodeInfo, OpaqueNode, TElement, TNode};
use style::properties::style_structs::Font;
use style::properties::{
    ComputedValues, Importance, LonghandId, PropertyDeclarationBlock, PropertyDeclarationId,
    PropertyId, ShorthandId, SourcePropertyDeclaration, parse_one_declaration_into,
};
use style::selector_parser::PseudoElement;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style::stylist::RuleInclusion;
use style::traversal::resolve_style;
use style::values::computed::{Float, Size};
use style::values::generics::font::LineHeight;
use style::values::generics::position::AspectRatio;
use style::values::specified::GenericGridTemplateComponent;
use style::values::specified::box_::DisplayInside;
use style::values::specified::text::TextTransformCase;
use style_traits::{ParsingMode, ToCss};

use crate::ArcRefCell;
use crate::dom::NodeExt;
use crate::flow::inline::construct::{TextTransformation, WhitespaceCollapse, capitalize_string};
use crate::fragment_tree::{
    BoxFragment, Fragment, FragmentFlags, FragmentTree, SpecificLayoutInfo,
};
use crate::taffy::SpecificTaffyGridInfo;

pub fn process_content_box_request(node: ServoLayoutNode<'_>) -> Option<Rect<Au>> {
    let rects: Vec<_> = node
        .fragments_for_pseudo(None)
        .iter()
        .filter_map(Fragment::cumulative_border_box_rect)
        .collect();
    if rects.is_empty() {
        return None;
    }

    Some(rects.iter().fold(Rect::zero(), |unioned_rect, rect| {
        rect.to_untyped().union(&unioned_rect)
    }))
}

pub fn process_content_boxes_request(node: ServoLayoutNode<'_>) -> Vec<Rect<Au>> {
    node.fragments_for_pseudo(None)
        .iter()
        .filter_map(Fragment::cumulative_border_box_rect)
        .map(|rect| rect.to_untyped())
        .collect()
}

pub fn process_client_rect_request(node: ServoLayoutNode<'_>) -> Rect<i32> {
    node.fragments_for_pseudo(None)
        .first()
        .map(Fragment::client_rect)
        .unwrap_or_default()
}

/// <https://drafts.csswg.org/cssom-view/#scrolling-area>
pub fn process_node_scroll_area_request(
    requested_node: Option<ServoLayoutNode<'_>>,
    fragment_tree: Option<Rc<FragmentTree>>,
) -> Rect<i32> {
    let Some(tree) = fragment_tree else {
        return Rect::zero();
    };

    let rect = match requested_node {
        Some(node) => node
            .fragments_for_pseudo(None)
            .first()
            .map(Fragment::scrolling_area)
            .unwrap_or_default(),
        None => tree.scrollable_overflow(),
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
pub fn process_resolved_style_request(
    context: &SharedStyleContext,
    node: ServoLayoutNode<'_>,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
) -> String {
    if !node.as_element().unwrap().has_data() {
        return process_resolved_style_request_for_unstyled_node(context, node, pseudo, property);
    }

    // We call process_resolved_style_request after performing a whole-document
    // traversal, so in the common case, the element is styled.
    let layout_element = node.to_threadsafe().as_element().unwrap();
    let layout_element = match pseudo {
        Some(pseudo_element_type) => {
            match layout_element.with_pseudo(*pseudo_element_type) {
                Some(layout_element) => layout_element,
                None => {
                    // The pseudo doesn't exist, return nothing.  Chrome seems to query
                    // the element itself in this case, Firefox uses the resolved value.
                    // https://www.w3.org/Bugs/Public/show_bug.cgi?id=29006
                    return String::new();
                },
            }
        },
        None => layout_element,
    };

    let style = &*layout_element.style(context);
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

    let computed_style = |fragment: Option<&Fragment>| match longhand_id {
        LonghandId::MinWidth
            if style.clone_min_width() == Size::Auto &&
                !should_honor_min_size_auto(fragment, style) =>
        {
            String::from("0px")
        },
        LonghandId::MinHeight
            if style.clone_min_height() == Size::Auto &&
                !should_honor_min_size_auto(fragment, style) =>
        {
            String::from("0px")
        },
        _ => style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id)),
    };

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
            // There could be a fragment, but it's only interesting for `min-width` and `min-height`,
            // so just pass None.
            LineHeight::Normal => computed_style(None),
            LineHeight::Number(value) => (font_size * value.0).to_css_string(),
            LineHeight::Length(value) => value.0.to_css_string(),
        };
    }

    // https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle
    // The properties that we calculate below all resolve to the computed value
    // when the element is display:none or display:contents.
    let display = style.get_box().display;
    if display.is_none() || display.is_contents() {
        return computed_style(None);
    }

    let resolve_for_fragment = |fragment: &Fragment| {
        let (content_rect, margins, padding, specific_layout_info) = match fragment {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => {
                let box_fragment = box_fragment.borrow();
                if style.get_box().position != Position::Static {
                    let resolved_insets = || box_fragment.calculate_resolved_insets_if_positioned();
                    match longhand_id {
                        LonghandId::Top => return resolved_insets().top.to_css_string(),
                        LonghandId::Right => {
                            return resolved_insets().right.to_css_string();
                        },
                        LonghandId::Bottom => {
                            return resolved_insets().bottom.to_css_string();
                        },
                        LonghandId::Left => {
                            return resolved_insets().left.to_css_string();
                        },
                        _ => {},
                    }
                }
                let content_rect = box_fragment.content_rect;
                let margins = box_fragment.margin;
                let padding = box_fragment.padding;
                let specific_layout_info = box_fragment.specific_layout_info.clone();
                (content_rect, margins, padding, specific_layout_info)
            },
            Fragment::Positioning(positioning_fragment) => {
                let content_rect = positioning_fragment.borrow().rect;
                (
                    content_rect,
                    SideOffsets2D::zero(),
                    SideOffsets2D::zero(),
                    None,
                )
            },
            _ => return computed_style(Some(fragment)),
        };

        // https://drafts.csswg.org/css-grid/#resolved-track-list
        // > The grid-template-rows and grid-template-columns properties are
        // > resolved value special case properties.
        //
        // > When an element generates a grid container box...
        if display.inside() == DisplayInside::Grid {
            if let Some(SpecificLayoutInfo::Grid(info)) = specific_layout_info {
                if let Some(value) = resolve_grid_template(&info, style, longhand_id) {
                    return value;
                }
            }
        }

        // https://drafts.csswg.org/cssom/#resolved-value-special-case-property-like-height
        // > If the property applies to the element or pseudo-element and the resolved value of the
        // > display property is not none or contents, then the resolved value is the used value.
        // > Otherwise the resolved value is the computed value.
        //
        // However, all browsers ignore that for margin and padding properties, and resolve to a length
        // even if the property doesn't apply: https://github.com/w3c/csswg-drafts/issues/10391
        match longhand_id {
            LonghandId::Width if resolved_size_should_be_used_value(fragment) => {
                content_rect.size.width
            },
            LonghandId::Height if resolved_size_should_be_used_value(fragment) => {
                content_rect.size.height
            },
            LonghandId::MarginBottom => margins.bottom,
            LonghandId::MarginTop => margins.top,
            LonghandId::MarginLeft => margins.left,
            LonghandId::MarginRight => margins.right,
            LonghandId::PaddingBottom => padding.bottom,
            LonghandId::PaddingTop => padding.top,
            LonghandId::PaddingLeft => padding.left,
            LonghandId::PaddingRight => padding.right,
            _ => return computed_style(Some(fragment)),
        }
        .to_css_string()
    };

    node.fragments_for_pseudo(*pseudo)
        .first()
        .map(resolve_for_fragment)
        .unwrap_or_else(|| computed_style(None))
}

fn resolved_size_should_be_used_value(fragment: &Fragment) -> bool {
    // https://drafts.csswg.org/css-sizing-3/#preferred-size-properties
    // > Applies to: all elements except non-replaced inlines
    match fragment {
        Fragment::Box(box_fragment) => !box_fragment.borrow().is_inline_box(),
        Fragment::Float(_) |
        Fragment::Positioning(_) |
        Fragment::AbsoluteOrFixedPositioned(_) |
        Fragment::Image(_) |
        Fragment::IFrame(_) => true,
        Fragment::Text(_) => false,
    }
}

fn should_honor_min_size_auto(fragment: Option<&Fragment>, style: &ComputedValues) -> bool {
    // <https://drafts.csswg.org/css-sizing-3/#automatic-minimum-size>
    // For backwards-compatibility, the resolved value of an automatic minimum size is zero
    // for boxes of all CSS2 display types: block and inline boxes, inline blocks, and all
    // the table layout boxes. It also resolves to zero when no box is generated.
    //
    // <https://github.com/w3c/csswg-drafts/issues/11716>
    // However, when a box is generated and `aspect-ratio` isn't `auto`, we need to preserve
    // the automatic minimum size as `auto`.
    let Some(Fragment::Box(box_fragment)) = fragment else {
        return false;
    };
    let flags = box_fragment.borrow().base.flags;
    flags.contains(FragmentFlags::IS_FLEX_OR_GRID_ITEM) ||
        style.clone_aspect_ratio() != AspectRatio::auto()
}

fn resolve_grid_template(
    grid_info: &SpecificTaffyGridInfo,
    style: &ComputedValues,
    longhand_id: LonghandId,
) -> Option<String> {
    // https://drafts.csswg.org/css-grid/#resolved-track-list-standalone
    fn serialize_standalone_non_subgrid_track_list(track_sizes: &[Au]) -> Option<String> {
        match track_sizes.is_empty() {
            // Standalone non subgrid grids with empty track lists should compute to `none`.
            // As of current standard, this behaviour should only invoked by `none` computed value,
            // therefore we can fallback into computed value resolving.
            true => None,
            // <https://drafts.csswg.org/css-grid/#resolved-track-list-standalone>
            // > - Every track listed individually, whether implicitly or explicitly created,
            //     without using the repeat() notation.
            // > - Every track size given as a length in pixels, regardless of sizing function.
            // > - Adjacent line names collapsed into a single bracketed set.
            // TODO: implement line names
            false => Some(
                track_sizes
                    .iter()
                    .map(|size| size.to_css_string())
                    .join(" "),
            ),
        }
    }

    let (track_info, computed_value) = match longhand_id {
        LonghandId::GridTemplateRows => (&grid_info.rows, &style.get_position().grid_template_rows),
        LonghandId::GridTemplateColumns => (
            &grid_info.columns,
            &style.get_position().grid_template_columns,
        ),
        _ => return None,
    };

    match computed_value {
        // <https://drafts.csswg.org/css-grid/#resolved-track-list-standalone>
        // > When an element generates a grid container box, the resolved value of its grid-template-rows or
        // > grid-template-columns property in a standalone axis is the used value, serialized with:
        GenericGridTemplateComponent::None |
        GenericGridTemplateComponent::TrackList(_) |
        GenericGridTemplateComponent::Masonry => {
            serialize_standalone_non_subgrid_track_list(&track_info.sizes)
        },

        // <https://drafts.csswg.org/css-grid/#resolved-track-list-subgrid>
        // > When an element generates a grid container box that is a subgrid, the resolved value of the
        // > grid-template-rows and grid-template-columns properties represents the used number of columns,
        // > serialized as the subgrid keyword followed by a list representing each of its lines as a
        // > line name set of all the line’s names explicitly defined on the subgrid (not including those
        // > adopted from the parent grid), without using the repeat() notation.
        // TODO: implement subgrid
        GenericGridTemplateComponent::Subgrid(_) => None,
    }
}

pub fn process_resolved_style_request_for_unstyled_node(
    context: &SharedStyleContext,
    node: ServoLayoutNode<'_>,
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

    match longhand_id {
        // <https://drafts.csswg.org/css-sizing-3/#automatic-minimum-size>
        // The resolved value of an automatic minimum size is zero when no box is generated.
        LonghandId::MinWidth if style.clone_min_width() == Size::Auto => String::from("0px"),
        LonghandId::MinHeight if style.clone_min_height() == Size::Auto => String::from("0px"),

        // No need to care about used values here, since we're on a display: none
        // subtree, use the computed value.
        _ => style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id)),
    }
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

struct OffsetParentFragments {
    parent: ArcRefCell<BoxFragment>,
    grandparent: Option<Fragment>,
}

/// <https://www.w3.org/TR/2016/WD-cssom-view-1-20160317/#dom-htmlelement-offsetparent>
fn offset_parent_fragments(node: ServoLayoutNode<'_>) -> Option<OffsetParentFragments> {
    // 1. If any of the following holds true return null and terminate this algorithm:
    //  * The element does not have an associated CSS layout box.
    //  * The element is the root element.
    //  * The element is the HTML body element.
    //  * The element’s computed value of the position property is fixed.
    let fragment = node.fragments_for_pseudo(None).first().cloned()?;
    let flags = fragment.base()?.flags;
    if flags.intersects(
        FragmentFlags::IS_ROOT_ELEMENT | FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT,
    ) {
        return None;
    }
    if matches!(
        fragment, Fragment::Box(fragment) if fragment.borrow().style.get_box().position == Position::Fixed
    ) {
        return None;
    }

    // 2.  Return the nearest ancestor element of the element for which at least one of
    //     the following is true and terminate this algorithm if such an ancestor is found:
    //  * The computed value of the position property is not static.
    //  * It is the HTML body element.
    //  * The computed value of the position property of the element is static and the
    //    ancestor is one of the following HTML elements: td, th, or table.
    let mut maybe_parent_node = node.parent_node();
    while let Some(parent_node) = maybe_parent_node {
        maybe_parent_node = parent_node.parent_node();

        if let Some(parent_fragment) = parent_node.fragments_for_pseudo(None).first() {
            let parent_fragment = match parent_fragment {
                Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => box_fragment,
                _ => continue,
            };

            let grandparent_fragment =
                maybe_parent_node.and_then(|node| node.fragments_for_pseudo(None).first().cloned());

            if parent_fragment.borrow().style.get_box().position != Position::Static {
                return Some(OffsetParentFragments {
                    parent: parent_fragment.clone(),
                    grandparent: grandparent_fragment,
                });
            }

            let flags = parent_fragment.borrow().base.flags;
            if flags.intersects(
                FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT |
                    FragmentFlags::IS_TABLE_TH_OR_TD_ELEMENT,
            ) {
                return Some(OffsetParentFragments {
                    parent: parent_fragment.clone(),
                    grandparent: grandparent_fragment,
                });
            }
        }
    }

    None
}

#[inline]
pub fn process_offset_parent_query(node: ServoLayoutNode<'_>) -> Option<OffsetParentResponse> {
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
    // > 1. If the element is the HTML body element or does not have any associated CSS
    //      layout box return zero and terminate this algorithm.
    let fragment = node.fragments_for_pseudo(None).first().cloned()?;
    let mut border_box = fragment.cumulative_border_box_rect()?;

    // 2.  If the offsetParent of the element is null return the x-coordinate of the left
    //     border edge of the first CSS layout box associated with the element, relative to
    //     the initial containing block origin, ignoring any transforms that apply to the
    //     element and its ancestors, and terminate this algorithm.
    let Some(offset_parent_fragment) = offset_parent_fragments(node) else {
        return Some(OffsetParentResponse {
            node_address: None,
            rect: border_box.to_untyped(),
        });
    };

    let parent_fragment = offset_parent_fragment.parent.borrow();
    let parent_is_static_body_element = parent_fragment
        .base
        .flags
        .contains(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT) &&
        parent_fragment.style.get_box().position == Position::Static;

    // For `offsetLeft`:
    // 3. Return the result of subtracting the y-coordinate of the top padding edge of the
    //    first CSS layout box associated with the offsetParent of the element from the
    //    y-coordinate of the top border edge of the first CSS layout box associated with the
    //    element, relative to the initial containing block origin, ignoring any transforms
    //    that apply to the element and its ancestors.
    //
    // We generalize this for `offsetRight` as described in the specification.
    let grandparent_box_fragment = || match offset_parent_fragment.grandparent {
        Some(Fragment::Box(box_fragment)) | Some(Fragment::Float(box_fragment)) => {
            Some(box_fragment)
        },
        _ => None,
    };

    // The spec (https://www.w3.org/TR/cssom-view-1/#extensions-to-the-htmlelement-interface)
    // says that offsetTop/offsetLeft are always relative to the padding box of the offsetParent.
    // However, in practice this is not true in major browsers in the case that the offsetParent is the body
    // element and the body element is position:static. In that case offsetLeft/offsetTop are computed
    // relative to the root node's border box.
    //
    // See <https://github.com/w3c/csswg-drafts/issues/10549>.
    let parent_offset_rect = if parent_is_static_body_element {
        if let Some(grandparent_fragment) = grandparent_box_fragment() {
            let grandparent_fragment = grandparent_fragment.borrow();
            grandparent_fragment.offset_by_containing_block(&grandparent_fragment.border_rect())
        } else {
            parent_fragment.offset_by_containing_block(&parent_fragment.padding_rect())
        }
    } else {
        parent_fragment.offset_by_containing_block(&parent_fragment.padding_rect())
    };

    border_box = border_box.translate(-parent_offset_rect.origin.to_vector());

    Some(OffsetParentResponse {
        node_address: parent_fragment.base.tag.map(|tag| tag.node.into()),
        rect: border_box.to_untyped(),
    })
}

/// <https://html.spec.whatwg.org/multipage/#get-the-text-steps>
pub fn get_the_text_steps(node: ServoLayoutNode<'_>) -> String {
    // Step 1: If element is not being rendered or if the user agent is a non-CSS user agent, then
    // return element's descendant text content.
    // This is taken care of in HTMLElemnent code

    // Step 2: Let results be a new empty list.
    let mut results = Vec::new();
    let mut max_req_line_break_count = 0;

    // Step 3: For each child node node of element:
    let mut state = Default::default();
    for child in node.dom_children() {
        // Step 1: Let current be the list resulting in running the rendered text collection steps with node.
        let mut current = rendered_text_collection_steps(child, &mut state);
        // Step 2: For each item item in current, append item to results.
        results.append(&mut current);
    }

    let mut output = Vec::new();
    for item in results {
        match item {
            InnerOrOuterTextItem::Text(s) => {
                // Step 3.
                if !s.is_empty() {
                    if max_req_line_break_count > 0 {
                        // Step 5.
                        output.push("\u{000A}".repeat(max_req_line_break_count));
                        max_req_line_break_count = 0;
                    }
                    output.push(s);
                }
            },
            InnerOrOuterTextItem::RequiredLineBreakCount(count) => {
                // Step 4.
                if output.is_empty() {
                    // Remove required line break count at the start.
                    continue;
                }
                // Store the count if it's the max of this run, but it may be ignored if no text
                // item is found afterwards, which means that these are consecutive line breaks at
                // the end.
                if count > max_req_line_break_count {
                    max_req_line_break_count = count;
                }
            },
        }
    }
    output.into_iter().collect()
}

enum InnerOrOuterTextItem {
    Text(String),
    RequiredLineBreakCount(usize),
}

#[derive(Clone)]
struct RenderedTextCollectionState {
    /// Used to make sure we don't add a `\n` before the first row
    first_table_row: bool,
    /// Used to make sure we don't add a `\t` before the first column
    first_table_cell: bool,
    /// Keeps track of whether we're inside a table, since there are special rules like ommiting everything that's not
    /// inside a TableCell/TableCaption
    within_table: bool,
    /// Determines whether we truncate leading whitespaces for normal nodes or not
    may_start_with_whitespace: bool,
    /// Is set whenever we truncated a white space char, used to prepend a single space before the next element,
    /// that way we truncate trailing white space without having to look ahead
    did_truncate_trailing_white_space: bool,
    /// Is set to true when we're rendering the children of TableCell/TableCaption elements, that way we render
    /// everything inside those as normal, while omitting everything that's in a Table but NOT in a Cell/Caption
    within_table_content: bool,
}

impl Default for RenderedTextCollectionState {
    fn default() -> Self {
        RenderedTextCollectionState {
            first_table_row: true,
            first_table_cell: true,
            may_start_with_whitespace: true,
            did_truncate_trailing_white_space: false,
            within_table: false,
            within_table_content: false,
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#rendered-text-collection-steps>
fn rendered_text_collection_steps(
    node: ServoLayoutNode<'_>,
    state: &mut RenderedTextCollectionState,
) -> Vec<InnerOrOuterTextItem> {
    // Step 1. Let items be the result of running the rendered text collection
    // steps with each child node of node in tree order,
    // and then concatenating the results to a single list.
    let mut items = vec![];
    if !node.is_connected() || !(node.is_element() || node.is_text_node()) {
        return items;
    }

    match node.type_id() {
        LayoutNodeType::Text => {
            if let Some(element) = node.parent_node() {
                match element.type_id() {
                    // Any text contained in these elements must be ignored.
                    LayoutNodeType::Element(LayoutElementType::HTMLCanvasElement) |
                    LayoutNodeType::Element(LayoutElementType::HTMLImageElement) |
                    LayoutNodeType::Element(LayoutElementType::HTMLIFrameElement) |
                    LayoutNodeType::Element(LayoutElementType::HTMLObjectElement) |
                    LayoutNodeType::Element(LayoutElementType::HTMLInputElement) |
                    LayoutNodeType::Element(LayoutElementType::HTMLTextAreaElement) |
                    LayoutNodeType::Element(LayoutElementType::HTMLMediaElement) => {
                        return items;
                    },
                    // Select/Option/OptGroup elements are handled a bit differently.
                    // Basically: a Select can only contain Options or OptGroups, while
                    // OptGroups may also contain Options. Everything else gets ignored.
                    LayoutNodeType::Element(LayoutElementType::HTMLOptGroupElement) => {
                        if let Some(element) = element.parent_node() {
                            if !matches!(
                                element.type_id(),
                                LayoutNodeType::Element(LayoutElementType::HTMLSelectElement)
                            ) {
                                return items;
                            }
                        } else {
                            return items;
                        }
                    },
                    LayoutNodeType::Element(LayoutElementType::HTMLSelectElement) => return items,
                    _ => {},
                }

                // Tables are also a bit special, mainly by only allowing
                // content within TableCell or TableCaption elements once
                // we're inside a Table.
                if state.within_table && !state.within_table_content {
                    return items;
                }

                let Some(style_data) = element.style_data() else {
                    return items;
                };

                let element_data = style_data.element_data.borrow();
                let Some(style) = element_data.styles.get_primary() else {
                    return items;
                };

                // Step 2: If node's computed value of 'visibility' is not 'visible', then return items.
                //
                // We need to do this check here on the Text fragment, if we did it on the element and
                // just skipped rendering all child nodes then there'd be no way to override the
                // visibility in a child node.
                if style.get_inherited_box().visibility != Visibility::Visible {
                    return items;
                }

                // Step 3: If node is not being rendered, then return items. For the purpose of this step,
                // the following elements must act as described if the computed value of the 'display'
                // property is not 'none':
                let display = style.get_box().display;
                if display == Display::None {
                    match element.type_id() {
                        // Even if set to Display::None, Option/OptGroup elements need to
                        // be rendered.
                        LayoutNodeType::Element(LayoutElementType::HTMLOptGroupElement) |
                        LayoutNodeType::Element(LayoutElementType::HTMLOptionElement) => {},
                        _ => {
                            return items;
                        },
                    }
                }

                let text_content = node.to_threadsafe().node_text_content();

                let white_space_collapse = style.clone_white_space_collapse();
                let preserve_whitespace = white_space_collapse == WhiteSpaceCollapseValue::Preserve;
                let is_inline = matches!(
                    display,
                    Display::InlineBlock | Display::InlineFlex | Display::InlineGrid
                );
                // Now we need to decide on whether to remove beginning white space or not, this
                // is mainly decided by the elements we rendered before, but may be overwritten by the white-space
                // property.
                let trim_beginning_white_space =
                    !preserve_whitespace && (state.may_start_with_whitespace || is_inline);
                let with_white_space_rules_applied = WhitespaceCollapse::new(
                    text_content.chars(),
                    white_space_collapse,
                    trim_beginning_white_space,
                );

                // Step 4: If node is a Text node, then for each CSS text box produced by node, in
                // content order, compute the text of the box after application of the CSS
                // 'white-space' processing rules and 'text-transform' rules, set items to the list
                // of the resulting strings, and return items. The CSS 'white-space' processing
                // rules are slightly modified: collapsible spaces at the end of lines are always
                // collapsed, but they are only removed if the line is the last line of the block,
                // or it ends with a br element. Soft hyphens should be preserved.
                let text_transform = style.clone_text_transform().case();
                let mut transformed_text: String =
                    TextTransformation::new(with_white_space_rules_applied, text_transform)
                        .collect();

                // Since iterator for capitalize not doing anything, we must handle it outside here
                // FIXME: This assumes the element always start at a word boundary. But can fail:
                // a<span style="text-transform: capitalize">b</span>c
                if TextTransformCase::Capitalize == text_transform {
                    transformed_text = capitalize_string(&transformed_text, true);
                }

                let is_preformatted_element =
                    white_space_collapse == WhiteSpaceCollapseValue::Preserve;

                let is_final_character_whitespace = transformed_text
                    .chars()
                    .next_back()
                    .filter(char::is_ascii_whitespace)
                    .is_some();

                let is_first_character_whitespace = transformed_text
                    .chars()
                    .next()
                    .filter(char::is_ascii_whitespace)
                    .is_some();

                // By truncating trailing white space and then adding it back in once we
                // encounter another text node we can ensure no trailing white space for
                // normal text without having to look ahead
                if state.did_truncate_trailing_white_space && !is_first_character_whitespace {
                    items.push(InnerOrOuterTextItem::Text(String::from(" ")));
                };

                if !transformed_text.is_empty() {
                    // Here we decide whether to keep or truncate the final white
                    // space character, if there is one.
                    if is_final_character_whitespace && !is_preformatted_element {
                        state.may_start_with_whitespace = false;
                        state.did_truncate_trailing_white_space = true;
                        transformed_text.pop();
                    } else {
                        state.may_start_with_whitespace = is_final_character_whitespace;
                        state.did_truncate_trailing_white_space = false;
                    }
                    items.push(InnerOrOuterTextItem::Text(transformed_text));
                }
            } else {
                // If we don't have a parent element then there's no style data available,
                // in this (pretty unlikely) case we just return the Text fragment as is.
                items.push(InnerOrOuterTextItem::Text(
                    node.to_threadsafe().node_text_content().into(),
                ));
            }
        },
        LayoutNodeType::Element(LayoutElementType::HTMLBRElement) => {
            // Step 5: If node is a br element, then append a string containing a single U+000A
            // LF code point to items.
            state.did_truncate_trailing_white_space = false;
            state.may_start_with_whitespace = true;
            items.push(InnerOrOuterTextItem::Text(String::from("\u{000A}")));
        },
        _ => {
            // First we need to gather some infos to setup the various flags
            // before rendering the child nodes
            let Some(style_data) = node.style_data() else {
                return items;
            };

            let element_data = style_data.element_data.borrow();
            let Some(style) = element_data.styles.get_primary() else {
                return items;
            };
            let inherited_box = style.get_inherited_box();

            if inherited_box.visibility != Visibility::Visible {
                // If the element is not visible then we'll immediatly render all children,
                // skipping all other processing.
                // We can't just stop here since a child can override a parents visibility.
                for child in node.dom_children() {
                    items.append(&mut rendered_text_collection_steps(child, state));
                }
                return items;
            }

            let style_box = style.get_box();
            let display = style_box.display;
            let mut surrounding_line_breaks = 0;

            // Treat absolutely positioned or floated elements like Block elements
            if style_box.position == Position::Absolute || style_box.float != Float::None {
                surrounding_line_breaks = 1;
            }

            // Depending on the display property we have to do various things
            // before we can render the child nodes.
            match display {
                Display::Table => {
                    surrounding_line_breaks = 1;
                    state.within_table = true;
                },
                // Step 6: If node's computed value of 'display' is 'table-cell',
                // and node's CSS box is not the last 'table-cell' box of its
                // enclosing 'table-row' box, then append a string containing
                // a single U+0009 TAB code point to items.
                Display::TableCell => {
                    if !state.first_table_cell {
                        items.push(InnerOrOuterTextItem::Text(String::from(
                            "\u{0009}", /* tab */
                        )));
                        // Make sure we don't add a white-space we removed from the previous node
                        state.did_truncate_trailing_white_space = false;
                    }
                    state.first_table_cell = false;
                    state.within_table_content = true;
                },
                // Step 7: If node's computed value of 'display' is 'table-row',
                // and node's CSS box is not the last 'table-row' box of the nearest
                // ancestor 'table' box, then append a string containing a single U+000A
                // LF code point to items.
                Display::TableRow => {
                    if !state.first_table_row {
                        items.push(InnerOrOuterTextItem::Text(String::from(
                            "\u{000A}", /* Line Feed */
                        )));
                        // Make sure we don't add a white-space we removed from the previous node
                        state.did_truncate_trailing_white_space = false;
                    }
                    state.first_table_row = false;
                    state.first_table_cell = true;
                },
                // Step 9: If node's used value of 'display' is block-level or 'table-caption',
                // then append 1 (a required line break count) at the beginning and end of items.
                Display::Block => {
                    surrounding_line_breaks = 1;
                },
                Display::TableCaption => {
                    surrounding_line_breaks = 1;
                    state.within_table_content = true;
                },
                Display::InlineFlex | Display::InlineGrid | Display::InlineBlock => {
                    // InlineBlock's are a bit strange, in that they don't produce a Linebreak, yet
                    // disable white space truncation before and after it, making it one of the few
                    // cases where one can have multiple white space characters following one another.
                    if state.did_truncate_trailing_white_space {
                        items.push(InnerOrOuterTextItem::Text(String::from(" ")));
                        state.did_truncate_trailing_white_space = false;
                        state.may_start_with_whitespace = true;
                    }
                },
                _ => {},
            }

            match node.type_id() {
                // Step 8: If node is a p element, then append 2 (a required line break count) at
                // the beginning and end of items.
                LayoutNodeType::Element(LayoutElementType::HTMLParagraphElement) => {
                    surrounding_line_breaks = 2;
                },
                // Option/OptGroup elements should go on separate lines, by treating them like
                // Block elements we can achieve that.
                LayoutNodeType::Element(LayoutElementType::HTMLOptionElement) |
                LayoutNodeType::Element(LayoutElementType::HTMLOptGroupElement) => {
                    surrounding_line_breaks = 1;
                },
                _ => {},
            }

            if surrounding_line_breaks > 0 {
                items.push(InnerOrOuterTextItem::RequiredLineBreakCount(
                    surrounding_line_breaks,
                ));
                state.did_truncate_trailing_white_space = false;
                state.may_start_with_whitespace = true;
            }

            match node.type_id() {
                // Any text/content contained in these elements is ignored.
                // However we still need to check whether we have to prepend a
                // space, since for example <span>asd <input> qwe</span> must
                // product "asd  qwe" (note the 2 spaces)
                LayoutNodeType::Element(LayoutElementType::HTMLCanvasElement) |
                LayoutNodeType::Element(LayoutElementType::HTMLImageElement) |
                LayoutNodeType::Element(LayoutElementType::HTMLIFrameElement) |
                LayoutNodeType::Element(LayoutElementType::HTMLObjectElement) |
                LayoutNodeType::Element(LayoutElementType::HTMLInputElement) |
                LayoutNodeType::Element(LayoutElementType::HTMLTextAreaElement) |
                LayoutNodeType::Element(LayoutElementType::HTMLMediaElement) => {
                    if display != Display::Block && state.did_truncate_trailing_white_space {
                        items.push(InnerOrOuterTextItem::Text(String::from(" ")));
                        state.did_truncate_trailing_white_space = false;
                    };
                    state.may_start_with_whitespace = false;
                },
                _ => {
                    // Now we can finally iterate over all children, appending whatever
                    // they produce to items.
                    for child in node.dom_children() {
                        items.append(&mut rendered_text_collection_steps(child, state));
                    }
                },
            }

            // Depending on the display property we still need to do some
            // cleanup after rendering all child nodes
            match display {
                Display::InlineFlex | Display::InlineGrid | Display::InlineBlock => {
                    state.did_truncate_trailing_white_space = false;
                    state.may_start_with_whitespace = false;
                },
                Display::Table => {
                    state.within_table = false;
                },
                Display::TableCell | Display::TableCaption => {
                    state.within_table_content = false;
                },
                _ => {},
            }

            if surrounding_line_breaks > 0 {
                items.push(InnerOrOuterTextItem::RequiredLineBreakCount(
                    surrounding_line_breaks,
                ));
                state.did_truncate_trailing_white_space = false;
                state.may_start_with_whitespace = true;
            }
        },
    };
    items
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
            node.to_threadsafe().as_element().unwrap().style(context)
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
