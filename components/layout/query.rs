/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by layout.

use std::cmp::{max, min};
use std::ops::Deref;

use app_units::Au;
use euclid::default::{Box2D, Point2D, Rect, Size2D, Vector2D};
use script_layout_interface::wrapper_traits::{
    LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_layout_interface::{LayoutElementType, LayoutNodeType, OffsetParentResponse};
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use style::computed_values::display::T as Display;
use style::computed_values::position::T as Position;
use style::computed_values::visibility::T as Visibility;
use style::context::{QuirksMode, SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use style::dom::TElement;
use style::logical_geometry::{BlockFlowDirection, InlineBaseDirection, WritingMode};
use style::properties::style_structs::{self, Font};
use style::properties::{
    parse_one_declaration_into, ComputedValues, Importance, LonghandId, PropertyDeclarationBlock,
    PropertyDeclarationId, PropertyId, ShorthandId, SourcePropertyDeclaration,
};
use style::selector_parser::PseudoElement;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style_traits::{ParsingMode, ToCss};

use crate::construct::ConstructionResult;
use crate::display_list::items::OpaqueNode;
use crate::display_list::IndexableText;
use crate::flow::{Flow, GetBaseFlow};
use crate::fragment::{Fragment, FragmentBorderBoxIterator, FragmentFlags, SpecificFragmentInfo};
use crate::inline::InlineFragmentNodeFlags;
use crate::sequential;
use crate::wrapper::ThreadSafeLayoutNodeHelpers;

// https://drafts.csswg.org/cssom-view/#overflow-directions
fn overflow_direction(writing_mode: &WritingMode) -> OverflowDirection {
    match (
        writing_mode.block_flow_direction(),
        writing_mode.inline_base_direction(),
    ) {
        (BlockFlowDirection::TopToBottom, InlineBaseDirection::LeftToRight) |
        (BlockFlowDirection::LeftToRight, InlineBaseDirection::LeftToRight) => {
            OverflowDirection::RightAndDown
        },
        (BlockFlowDirection::TopToBottom, InlineBaseDirection::RightToLeft) |
        (BlockFlowDirection::RightToLeft, InlineBaseDirection::LeftToRight) => {
            OverflowDirection::LeftAndDown
        },
        (BlockFlowDirection::RightToLeft, InlineBaseDirection::RightToLeft) => {
            OverflowDirection::LeftAndUp
        },
        (BlockFlowDirection::LeftToRight, InlineBaseDirection::RightToLeft) => {
            OverflowDirection::RightAndUp
        },
    }
}

struct UnioningFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    rect: Option<Rect<Au>>,
}

impl UnioningFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> UnioningFragmentBorderBoxIterator {
        UnioningFragmentBorderBoxIterator {
            node_address,
            rect: None,
        }
    }
}

impl FragmentBorderBoxIterator for UnioningFragmentBorderBoxIterator {
    fn process(&mut self, _: &Fragment, _: i32, border_box: &Rect<Au>) {
        self.rect = match self.rect {
            Some(rect) => Some(rect.union(border_box)),
            None => Some(*border_box),
        };
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

struct CollectingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    rects: Vec<Rect<Au>>,
}

impl CollectingFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> CollectingFragmentBorderBoxIterator {
        CollectingFragmentBorderBoxIterator {
            node_address,
            rects: Vec::new(),
        }
    }
}

impl FragmentBorderBoxIterator for CollectingFragmentBorderBoxIterator {
    fn process(&mut self, _: &Fragment, _: i32, border_box: &Rect<Au>) {
        self.rects.push(*border_box);
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

enum Side {
    Left,
    Right,
    Bottom,
    Top,
}

enum MarginPadding {
    Margin,
    Padding,
}

enum PositionProperty {
    Left,
    Right,
    Top,
    Bottom,
    Width,
    Height,
}

#[derive(Debug)]
enum OverflowDirection {
    RightAndDown,
    LeftAndDown,
    LeftAndUp,
    RightAndUp,
}

struct PositionRetrievingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    result: Option<Au>,
    position: Point2D<Au>,
    property: PositionProperty,
}

impl PositionRetrievingFragmentBorderBoxIterator {
    fn new(
        node_address: OpaqueNode,
        property: PositionProperty,
        position: Point2D<Au>,
    ) -> PositionRetrievingFragmentBorderBoxIterator {
        PositionRetrievingFragmentBorderBoxIterator {
            node_address,
            position,
            property,
            result: None,
        }
    }
}

impl FragmentBorderBoxIterator for PositionRetrievingFragmentBorderBoxIterator {
    fn process(&mut self, fragment: &Fragment, _: i32, border_box: &Rect<Au>) {
        let border_padding = fragment
            .border_padding
            .to_physical(fragment.style.writing_mode);
        self.result = Some(match self.property {
            PositionProperty::Left => self.position.x,
            PositionProperty::Top => self.position.y,
            PositionProperty::Width => border_box.size.width - border_padding.horizontal(),
            PositionProperty::Height => border_box.size.height - border_padding.vertical(),
            // TODO: the following 2 calculations are completely wrong.
            // They should return the difference between the parent's and this
            // fragment's border boxes.
            PositionProperty::Right => border_box.max_x() + self.position.x,
            PositionProperty::Bottom => border_box.max_y() + self.position.y,
        });
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

struct MarginRetrievingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    result: Option<Au>,
    writing_mode: WritingMode,
    margin_padding: MarginPadding,
    side: Side,
}

impl MarginRetrievingFragmentBorderBoxIterator {
    fn new(
        node_address: OpaqueNode,
        side: Side,
        margin_padding: MarginPadding,
        writing_mode: WritingMode,
    ) -> MarginRetrievingFragmentBorderBoxIterator {
        MarginRetrievingFragmentBorderBoxIterator {
            node_address,
            side,
            margin_padding,
            result: None,
            writing_mode,
        }
    }
}

impl FragmentBorderBoxIterator for MarginRetrievingFragmentBorderBoxIterator {
    fn process(&mut self, fragment: &Fragment, _: i32, _: &Rect<Au>) {
        let rect = match self.margin_padding {
            MarginPadding::Margin => &fragment.margin,
            MarginPadding::Padding => &fragment.border_padding,
        };
        self.result = Some(match self.side {
            Side::Left => rect.left(self.writing_mode),
            Side::Right => rect.right(self.writing_mode),
            Side::Bottom => rect.bottom(self.writing_mode),
            Side::Top => rect.top(self.writing_mode),
        });
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

pub fn process_content_box_request(
    requested_node: OpaqueNode,
    layout_root: &mut dyn Flow,
) -> Option<Rect<Au>> {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let mut iterator = UnioningFragmentBorderBoxIterator::new(requested_node);
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.rect
}

pub fn process_content_boxes_request(
    requested_node: OpaqueNode,
    layout_root: &mut dyn Flow,
) -> Vec<Rect<Au>> {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let mut iterator = CollectingFragmentBorderBoxIterator::new(requested_node);
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.rects
}

struct FragmentClientRectQueryIterator {
    node_address: OpaqueNode,
    client_rect: Rect<i32>,
}

impl FragmentClientRectQueryIterator {
    fn new(node_address: OpaqueNode) -> FragmentClientRectQueryIterator {
        FragmentClientRectQueryIterator {
            node_address,
            client_rect: Rect::zero(),
        }
    }
}

struct UnioningFragmentScrollAreaIterator {
    node_address: OpaqueNode,
    union_rect: Rect<i32>,
    origin_rect: Rect<i32>,
    level: Option<i32>,
    is_child: bool,
    overflow_direction: OverflowDirection,
}

impl UnioningFragmentScrollAreaIterator {
    fn new(node_address: OpaqueNode) -> UnioningFragmentScrollAreaIterator {
        UnioningFragmentScrollAreaIterator {
            node_address,
            union_rect: Rect::zero(),
            origin_rect: Rect::zero(),
            level: None,
            is_child: false,
            // FIXME(#20867)
            overflow_direction: OverflowDirection::RightAndDown,
        }
    }
}

struct NodeOffsetBoxInfo {
    offset: Point2D<Au>,
    rectangle: Rect<Au>,
}

struct ParentBorderBoxInfo {
    node_address: OpaqueNode,
    origin: Point2D<Au>,
}

struct ParentOffsetBorderBoxIterator {
    node_address: OpaqueNode,
    has_processed_node: bool,
    node_offset_box: Option<NodeOffsetBoxInfo>,
    parent_nodes: Vec<Option<ParentBorderBoxInfo>>,
}

impl ParentOffsetBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> ParentOffsetBorderBoxIterator {
        ParentOffsetBorderBoxIterator {
            node_address,
            has_processed_node: false,
            node_offset_box: None,
            parent_nodes: Vec::new(),
        }
    }
}

impl FragmentBorderBoxIterator for FragmentClientRectQueryIterator {
    fn process(&mut self, fragment: &Fragment, _: i32, border_box: &Rect<Au>) {
        let style_structs::Border {
            border_top_width: top_width,
            border_right_width: right_width,
            border_bottom_width: bottom_width,
            border_left_width: left_width,
            ..
        } = *fragment.style.get_border();
        let (left_width, right_width) = (left_width.to_px(), right_width.to_px());
        let (top_width, bottom_width) = (top_width.to_px(), bottom_width.to_px());
        self.client_rect.origin.y = top_width;
        self.client_rect.origin.x = left_width;
        self.client_rect.size.width = border_box.size.width.to_px() - left_width - right_width;
        self.client_rect.size.height = border_box.size.height.to_px() - top_width - bottom_width;
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.node == self.node_address
    }
}

// https://drafts.csswg.org/cssom-view/#scrolling-area
impl FragmentBorderBoxIterator for UnioningFragmentScrollAreaIterator {
    fn process(&mut self, fragment: &Fragment, level: i32, border_box: &Rect<Au>) {
        // In cases in which smaller child elements contain less padding than the parent
        // the a union of the two elements padding rectangles could result in an unwanted
        // increase in size. To work around this, we store the original elements padding
        // rectangle as `origin_rect` and the union of all child elements padding and
        // margin rectangles as `union_rect`.
        let style_structs::Border {
            border_top_width: top_border,
            border_right_width: right_border,
            border_bottom_width: bottom_border,
            border_left_width: left_border,
            ..
        } = *fragment.style.get_border();
        let (left_border, right_border) = (left_border.to_px(), right_border.to_px());
        let (top_border, bottom_border) = (top_border.to_px(), bottom_border.to_px());
        let right_padding = border_box.size.width.to_px() - right_border - left_border;
        let bottom_padding = border_box.size.height.to_px() - bottom_border - top_border;
        let top_padding = top_border;
        let left_padding = left_border;

        match self.level {
            Some(start_level) if level <= start_level => {
                self.is_child = false;
            },
            Some(_) => {
                let padding = Rect::new(
                    Point2D::new(left_padding, top_padding),
                    Size2D::new(right_padding, bottom_padding),
                );
                let top_margin = fragment.margin.top(fragment.style.writing_mode).to_px();
                let left_margin = fragment.margin.left(fragment.style.writing_mode).to_px();
                let bottom_margin = fragment.margin.bottom(fragment.style.writing_mode).to_px();
                let right_margin = fragment.margin.right(fragment.style.writing_mode).to_px();
                let margin = Rect::new(
                    Point2D::new(left_margin, top_margin),
                    Size2D::new(right_margin, bottom_margin),
                );

                // This is a workaround because euclid does not support unioning empty
                // rectangles.
                // TODO: The way that this iterator is calculating scroll area is very
                // suspect and the code below is a workaround until it can be written
                // in a better way.
                self.union_rect = Box2D::new(
                    Point2D::new(
                        min(
                            padding.min_x(),
                            min(margin.min_x(), self.union_rect.min_x()),
                        ),
                        min(
                            padding.min_y(),
                            min(margin.min_y(), self.union_rect.min_y()),
                        ),
                    ),
                    Point2D::new(
                        max(
                            padding.max_x(),
                            max(margin.max_x(), self.union_rect.max_x()),
                        ),
                        max(
                            padding.max_y(),
                            max(margin.max_y(), self.union_rect.max_y()),
                        ),
                    ),
                )
                .to_rect();
            },
            None => {
                self.level = Some(level);
                self.is_child = true;
                self.overflow_direction = overflow_direction(&fragment.style.writing_mode);
                self.origin_rect = Rect::new(
                    Point2D::new(left_padding, top_padding),
                    Size2D::new(right_padding, bottom_padding),
                );
            },
        };
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address) || self.is_child
    }
}

// https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
impl FragmentBorderBoxIterator for ParentOffsetBorderBoxIterator {
    fn process(&mut self, fragment: &Fragment, level: i32, border_box: &Rect<Au>) {
        if self.node_offset_box.is_none() {
            // We haven't found the node yet, so we're still looking
            // for its parent. Remove all nodes at this level or
            // higher, as they can't be parents of this node.
            self.parent_nodes.truncate(level as usize);
            assert_eq!(
                self.parent_nodes.len(),
                level as usize,
                "Skipped at least one level in the flow tree!"
            );
        }

        if !fragment.is_primary_fragment() {
            // This fragment doesn't correspond to anything worth
            // taking measurements from.

            if self.node_offset_box.is_none() {
                // If this is the only fragment in the flow, we need to
                // do this to avoid failing the above assertion.
                self.parent_nodes.push(None);
            }

            return;
        }

        if fragment.node == self.node_address {
            // Found the fragment in the flow tree that matches the
            // DOM node being looked for.

            assert!(
                self.node_offset_box.is_none(),
                "Node was being treated as inline, but it has an associated fragment!"
            );

            self.has_processed_node = true;
            self.node_offset_box = Some(NodeOffsetBoxInfo {
                offset: border_box.origin,
                rectangle: *border_box,
            });

            // offsetParent returns null if the node is fixed.
            if fragment.style.get_box().position == Position::Fixed {
                self.parent_nodes.clear();
            }
        } else if let Some(node) = fragment.inline_context.as_ref().and_then(|inline_context| {
            inline_context
                .nodes
                .iter()
                .find(|node| node.address == self.node_address)
        }) {
            // TODO: Handle cases where the `offsetParent` is an inline
            // element. This will likely be impossible until
            // https://github.com/servo/servo/issues/13982 is fixed.

            // Found a fragment in the flow tree whose inline context
            // contains the DOM node we're looking for, i.e. the node
            // is inline and contains this fragment.
            match self.node_offset_box {
                Some(NodeOffsetBoxInfo {
                    ref mut rectangle, ..
                }) => {
                    *rectangle = rectangle.union(border_box);
                },
                None => {
                    // https://github.com/servo/servo/issues/13982 will
                    // cause this assertion to fail sometimes, so it's
                    // commented out for now.
                    /*assert!(node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT),
                    "First fragment of inline node found wasn't its first fragment!");*/

                    self.node_offset_box = Some(NodeOffsetBoxInfo {
                        offset: border_box.origin,
                        rectangle: *border_box,
                    });
                },
            }

            if node
                .flags
                .contains(InlineFragmentNodeFlags::LAST_FRAGMENT_OF_ELEMENT)
            {
                self.has_processed_node = true;
            }
        } else if self.node_offset_box.is_none() {
            let is_body_element = fragment
                .flags
                .contains(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT);
            let is_valid_parent = match (
                is_body_element,
                fragment.style.get_box().position,
                &fragment.specific,
            ) {
                // Spec says it's valid if any of these are true:
                //  1) Is the body element
                //  2) Is static position *and* is a table or table cell
                //  3) Is not static position
                (true, _, _) |
                (false, Position::Static, &SpecificFragmentInfo::Table) |
                (false, Position::Static, &SpecificFragmentInfo::TableCell) |
                (false, Position::Sticky, _) |
                (false, Position::Absolute, _) |
                (false, Position::Relative, _) |
                (false, Position::Fixed, _) => true,

                // Otherwise, it's not a valid parent
                (false, Position::Static, _) => false,
            };

            let parent_info = if is_valid_parent {
                let border_width = fragment
                    .border_width()
                    .to_physical(fragment.style.writing_mode);

                Some(ParentBorderBoxInfo {
                    node_address: fragment.node,
                    origin: border_box.origin + Vector2D::new(border_width.left, border_width.top),
                })
            } else {
                None
            };

            self.parent_nodes.push(parent_info);
        }
    }

    fn should_process(&mut self, _: &Fragment) -> bool {
        !self.has_processed_node
    }
}

pub fn process_client_rect_query(
    requested_node: OpaqueNode,
    layout_root: &mut dyn Flow,
) -> Rect<i32> {
    let mut iterator = FragmentClientRectQueryIterator::new(requested_node);
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.client_rect
}

/// <https://drafts.csswg.org/cssom-view/#scrolling-area>
pub fn process_scrolling_area_request(
    requested_node: Option<OpaqueNode>,
    layout_root: &mut dyn Flow,
) -> Rect<i32> {
    let requested_node = match requested_node {
        Some(node) => node,
        None => {
            let rect = layout_root.base().overflow.scroll;
            return Rect::new(
                Point2D::new(rect.origin.x.to_nearest_px(), rect.origin.y.to_nearest_px()),
                Size2D::new(rect.width().ceil_to_px(), rect.height().ceil_to_px()),
            );
        },
    };

    let mut iterator = UnioningFragmentScrollAreaIterator::new(requested_node);
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    match iterator.overflow_direction {
        OverflowDirection::RightAndDown => {
            let right = max(
                iterator.union_rect.size.width,
                iterator.origin_rect.size.width,
            );
            let bottom = max(
                iterator.union_rect.size.height,
                iterator.origin_rect.size.height,
            );
            Rect::new(iterator.origin_rect.origin, Size2D::new(right, bottom))
        },
        OverflowDirection::LeftAndDown => {
            let bottom = max(
                iterator.union_rect.size.height,
                iterator.origin_rect.size.height,
            );
            let left = min(iterator.union_rect.origin.x, iterator.origin_rect.origin.x);
            Rect::new(
                Point2D::new(left, iterator.origin_rect.origin.y),
                Size2D::new(iterator.origin_rect.size.width, bottom),
            )
        },
        OverflowDirection::LeftAndUp => {
            let top = min(iterator.union_rect.origin.y, iterator.origin_rect.origin.y);
            let left = min(iterator.union_rect.origin.x, iterator.origin_rect.origin.x);
            Rect::new(Point2D::new(left, top), iterator.origin_rect.size)
        },
        OverflowDirection::RightAndUp => {
            let top = min(iterator.union_rect.origin.y, iterator.origin_rect.origin.y);
            let right = max(
                iterator.union_rect.size.width,
                iterator.origin_rect.size.width,
            );
            Rect::new(
                Point2D::new(iterator.origin_rect.origin.x, top),
                Size2D::new(right, iterator.origin_rect.size.height),
            )
        },
    }
}

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

pub fn process_resolved_font_style_request<'dom, E>(
    context: &SharedStyleContext,
    node: E,
    value: &str,
    url_data: ServoUrl,
    shared_lock: &SharedRwLock,
) -> Option<ServoArc<Font>>
where
    E: LayoutNode<'dom>,
{
    use style::stylist::RuleInclusion;
    use style::traversal::resolve_style;

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

/// Return the resolved value of property for a given (pseudo)element.
/// <https://drafts.csswg.org/cssom/#resolved-value>
pub fn process_resolved_style_request<'dom>(
    context: &SharedStyleContext,
    node: impl LayoutNode<'dom>,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
    layout_root: &mut dyn Flow,
) -> String {
    use style::stylist::RuleInclusion;
    use style::traversal::resolve_style;

    let element = node.as_element().unwrap();

    // We call process_resolved_style_request after performing a whole-document
    // traversal, so in the common case, the element is styled.
    if element.has_data() {
        return process_resolved_style_request_internal(node, pseudo, property, layout_root);
    }

    // In a display: none subtree. No pseudo-element exists.
    if pseudo.is_some() {
        return String::new();
    }

    let mut tlc = ThreadLocalStyleContext::new();
    let mut context = StyleContext {
        shared: context,
        thread_local: &mut tlc,
    };

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

/// The primary resolution logic, which assumes that the element is styled.
fn process_resolved_style_request_internal<'dom>(
    requested_node: impl LayoutNode<'dom>,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
    layout_root: &mut dyn Flow,
) -> String {
    let layout_el = requested_node.to_threadsafe().as_element().unwrap();
    let layout_el = match *pseudo {
        Some(PseudoElement::Before) => layout_el.get_before_pseudo(),
        Some(PseudoElement::After) => layout_el.get_after_pseudo(),
        Some(PseudoElement::DetailsSummary) |
        Some(PseudoElement::DetailsContent) |
        Some(PseudoElement::Selection) => None,
        // FIXME(emilio): What about the other pseudos? Probably they shouldn't
        // just return the element's style!
        _ => Some(layout_el),
    };

    let layout_el = match layout_el {
        None => {
            // The pseudo doesn't exist, return nothing.  Chrome seems to query
            // the element itself in this case, Firefox uses the resolved value.
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=29006
            return String::new();
        },
        Some(layout_el) => layout_el,
    };

    let style = &*layout_el.resolved_style();
    let longhand_id = match *property {
        PropertyId::NonCustom(id) => match id.longhand_or_shorthand() {
            Ok(longhand_id) => longhand_id,
            Err(shorthand_id) => return shorthand_to_css_string(shorthand_id, style),
        },
        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name));
        },
    };

    let positioned = matches!(
        style.get_box().position,
        Position::Relative | Position::Sticky | Position::Fixed | Position::Absolute
    );

    //TODO: determine whether requested property applies to the element.
    //      eg. width does not apply to non-replaced inline elements.
    // Existing browsers disagree about when left/top/right/bottom apply
    // (Chrome seems to think they never apply and always returns resolved values).
    // There are probably other quirks.
    let applies = true;

    fn used_value_for_position_property<'dom, N>(
        layout_el: <N::ConcreteThreadSafeLayoutNode as ThreadSafeLayoutNode<'dom>>::ConcreteThreadSafeLayoutElement,
        layout_root: &mut dyn Flow,
        requested_node: N,
        longhand_id: LonghandId,
    ) -> String
    where
        N: LayoutNode<'dom>,
    {
        let maybe_data = layout_el.as_node().borrow_layout_data();
        let position = maybe_data.map_or(Point2D::zero(), |data| {
            match data.flow_construction_result {
                ConstructionResult::Flow(ref flow_ref, _) => flow_ref
                    .deref()
                    .base()
                    .stacking_relative_position
                    .to_point(),
                // TODO(dzbarsky) search parents until we find node with a flow ref.
                // https://github.com/servo/servo/issues/8307
                _ => Point2D::zero(),
            }
        });
        let property = match longhand_id {
            LonghandId::Bottom => PositionProperty::Bottom,
            LonghandId::Top => PositionProperty::Top,
            LonghandId::Left => PositionProperty::Left,
            LonghandId::Right => PositionProperty::Right,
            LonghandId::Width => PositionProperty::Width,
            LonghandId::Height => PositionProperty::Height,
            _ => unreachable!(),
        };
        let mut iterator = PositionRetrievingFragmentBorderBoxIterator::new(
            requested_node.opaque(),
            property,
            position,
        );
        sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
        iterator
            .result
            .map(|r| r.to_css_string())
            .unwrap_or_default()
    }

    // TODO: we will return neither the computed nor used value for margin and padding.
    match longhand_id {
        LonghandId::MarginBottom |
        LonghandId::MarginTop |
        LonghandId::MarginLeft |
        LonghandId::MarginRight |
        LonghandId::PaddingBottom |
        LonghandId::PaddingTop |
        LonghandId::PaddingLeft |
        LonghandId::PaddingRight
            if applies && style.get_box().display != Display::None =>
        {
            let (margin_padding, side) = match longhand_id {
                LonghandId::MarginBottom => (MarginPadding::Margin, Side::Bottom),
                LonghandId::MarginTop => (MarginPadding::Margin, Side::Top),
                LonghandId::MarginLeft => (MarginPadding::Margin, Side::Left),
                LonghandId::MarginRight => (MarginPadding::Margin, Side::Right),
                LonghandId::PaddingBottom => (MarginPadding::Padding, Side::Bottom),
                LonghandId::PaddingTop => (MarginPadding::Padding, Side::Top),
                LonghandId::PaddingLeft => (MarginPadding::Padding, Side::Left),
                LonghandId::PaddingRight => (MarginPadding::Padding, Side::Right),
                _ => unreachable!(),
            };
            let mut iterator = MarginRetrievingFragmentBorderBoxIterator::new(
                requested_node.opaque(),
                side,
                margin_padding,
                style.writing_mode,
            );
            sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
            iterator
                .result
                .map(|r| r.to_css_string())
                .unwrap_or_default()
        },

        LonghandId::Bottom | LonghandId::Top | LonghandId::Right | LonghandId::Left
            if applies && positioned && style.get_box().display != Display::None =>
        {
            used_value_for_position_property(layout_el, layout_root, requested_node, longhand_id)
        },
        LonghandId::Width | LonghandId::Height
            if applies && style.get_box().display != Display::None =>
        {
            used_value_for_position_property(layout_el, layout_root, requested_node, longhand_id)
        },
        // FIXME: implement used value computation for line-height
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

pub fn process_offset_parent_query(
    requested_node: OpaqueNode,
    layout_root: &mut dyn Flow,
) -> OffsetParentResponse {
    let mut iterator = ParentOffsetBorderBoxIterator::new(requested_node);
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);

    let node_offset_box = iterator.node_offset_box;
    let parent_info = iterator.parent_nodes.into_iter().rev().flatten().next();
    match (node_offset_box, parent_info) {
        (Some(node_offset_box), Some(parent_info)) => {
            let origin = node_offset_box.offset - parent_info.origin.to_vector();
            let size = node_offset_box.rectangle.size;
            OffsetParentResponse {
                node_address: Some(parent_info.node_address.into()),
                rect: Rect::new(origin, size),
            }
        },
        _ => OffsetParentResponse::default(),
    }
}

enum InnerTextItem {
    Text(String),
    RequiredLineBreakCount(u32),
}

// https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute
pub fn process_element_inner_text_query<'dom>(
    node: impl LayoutNode<'dom>,
    indexable_text: &IndexableText,
) -> String {
    // Step 1.
    let mut results = Vec::new();
    // Step 2.
    inner_text_collection_steps(node, indexable_text, &mut results);
    let mut max_req_line_break_count = 0;
    let mut inner_text = Vec::new();
    for item in results {
        match item {
            InnerTextItem::Text(s) => {
                if max_req_line_break_count > 0 {
                    // Step 5.
                    for _ in 0..max_req_line_break_count {
                        inner_text.push("\u{000A}".to_owned());
                    }
                    max_req_line_break_count = 0;
                }
                // Step 3.
                if !s.is_empty() {
                    inner_text.push(s.to_owned());
                }
            },
            InnerTextItem::RequiredLineBreakCount(count) => {
                // Step 4.
                if inner_text.is_empty() {
                    // Remove required line break count at the start.
                    continue;
                }
                // Store the count if it's the max of this run,
                // but it may be ignored if no text item is found afterwards,
                // which means that these are consecutive line breaks at the end.
                if count > max_req_line_break_count {
                    max_req_line_break_count = count;
                }
            },
        }
    }
    inner_text.into_iter().collect()
}

// https://html.spec.whatwg.org/multipage/#inner-text-collection-steps
#[allow(unsafe_code)]
fn inner_text_collection_steps<'dom>(
    node: impl LayoutNode<'dom>,
    indexable_text: &IndexableText,
    results: &mut Vec<InnerTextItem>,
) {
    let mut items = Vec::new();
    for child in node.traverse_preorder() {
        let node = match child.type_id() {
            LayoutNodeType::Text => child.parent_node().unwrap(),
            _ => child,
        };

        let element_data = match node.style_data() {
            Some(data) => &data.element_data,
            None => continue,
        };

        let style = match element_data.borrow().styles.get_primary() {
            None => continue,
            Some(style) => style.clone(),
        };

        // Step 2.
        if style.get_inherited_box().visibility != Visibility::Visible {
            continue;
        }

        // Step 3.
        let display = style.get_box().display;
        if !child.is_connected() || display == Display::None {
            continue;
        }

        match child.type_id() {
            LayoutNodeType::Text => {
                // Step 4.
                if let Some(text_content) = indexable_text.get(child.opaque()) {
                    for content in text_content {
                        items.push(InnerTextItem::Text(content.text_run.text.to_string()));
                    }
                }
            },
            LayoutNodeType::Element(LayoutElementType::HTMLBRElement) => {
                // Step 5.
                items.push(InnerTextItem::Text(String::from(
                    "\u{000A}", /* line feed */
                )));
            },
            LayoutNodeType::Element(LayoutElementType::HTMLParagraphElement) => {
                // Step 8.
                items.insert(0, InnerTextItem::RequiredLineBreakCount(2));
                items.push(InnerTextItem::RequiredLineBreakCount(2));
            },
            _ => {},
        }

        match display {
            Display::TableCell if !is_last_table_cell() => {
                // Step 6.
                items.push(InnerTextItem::Text(String::from("\u{0009}" /* tab */)));
            },
            Display::TableRow if !is_last_table_row() => {
                // Step 7.
                items.push(InnerTextItem::Text(String::from(
                    "\u{000A}", /* line feed */
                )));
            },
            Display::Block | Display::Flex | Display::TableCaption | Display::Table => {
                // Step 9.
                items.insert(0, InnerTextItem::RequiredLineBreakCount(1));
                items.push(InnerTextItem::RequiredLineBreakCount(1));
            },
            _ => {},
        }
    }

    results.append(&mut items);
}

fn is_last_table_cell() -> bool {
    // FIXME(ferjm) Implement this.
    false
}

fn is_last_table_row() -> bool {
    // FIXME(ferjm) Implement this.
    false
}
