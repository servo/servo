/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeExt};
use crate::flow::construct::ContainsFloats;
use crate::flow::float::FloatBox;
use crate::flow::{BlockContainer, BlockFormattingContext, BlockLevelBox};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::Fragment;
use crate::geom::flow_relative::Vec2;
use crate::geom::{PhysicalPoint, PhysicalRect, PhysicalSize};
use crate::positioned::AbsolutelyPositionedBox;
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContent;
use crate::sizing::ContentSizesRequest;
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside};
use crate::DefiniteContainingBlock;
use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use gfx_traits::print_tree::PrintTree;
use script_layout_interface::wrapper_traits::LayoutNode;
use servo_arc::Arc;
use style::dom::OpaqueNode;
use style::properties::{style_structs, ComputedValues};
use style::values::computed::Length;
use style_traits::CSSPixel;

pub struct BoxTreeRoot(BlockFormattingContext);

pub struct FragmentTreeRoot {
    /// The children of the root of the fragment tree.
    children: Vec<Fragment>,

    /// The scrollable overflow of the root of the fragment tree.
    scrollable_overflow: PhysicalRect<Length>,

    /// The containing block used in the layout of this fragment tree.
    initial_containing_block: PhysicalRect<Length>,
}

impl BoxTreeRoot {
    pub fn construct<'dom, Node>(context: &LayoutContext, root_element: Node) -> Self
    where
        Node: 'dom + Copy + LayoutNode + Send + Sync,
    {
        let (contains_floats, boxes) = construct_for_root_element(&context, root_element);
        Self(BlockFormattingContext {
            contains_floats: contains_floats == ContainsFloats::Yes,
            contents: BlockContainer::BlockLevelBoxes(boxes),
        })
    }
}

fn construct_for_root_element<'dom>(
    context: &LayoutContext,
    root_element: impl NodeExt<'dom>,
) -> (ContainsFloats, Vec<Arc<BlockLevelBox>>) {
    let style = root_element.style(context);
    let replaced = ReplacedContent::for_element(root_element);
    let box_style = style.get_box();

    let display_inside = match Display::from(box_style.display) {
        Display::None => return (ContainsFloats::No, Vec::new()),
        Display::Contents if replaced.is_some() => {
            // 'display: contents' computes to 'none' for replaced elements
            return (ContainsFloats::No, Vec::new());
        },
        // https://drafts.csswg.org/css-display-3/#transformations
        Display::Contents => DisplayInside::Flow,
        // The root element is blockified, ignore DisplayOutside
        Display::GeneratingBox(DisplayGeneratingBox::OutsideInside { inside, .. }) => inside,
    };

    let contents = replaced.map_or(Contents::OfElement, Contents::Replaced);
    if box_style.position.is_absolutely_positioned() {
        (
            ContainsFloats::No,
            vec![Arc::new(BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(
                AbsolutelyPositionedBox::construct(
                    context,
                    root_element,
                    style,
                    display_inside,
                    contents,
                ),
            ))],
        )
    } else if box_style.float.is_floating() {
        (
            ContainsFloats::Yes,
            vec![Arc::new(BlockLevelBox::OutOfFlowFloatBox(
                FloatBox::construct(context, root_element, style, display_inside, contents),
            ))],
        )
    } else {
        (
            ContainsFloats::No,
            vec![Arc::new(BlockLevelBox::Independent(
                IndependentFormattingContext::construct(
                    context,
                    root_element,
                    style,
                    display_inside,
                    contents,
                    ContentSizesRequest::None,
                ),
            ))],
        )
    }
}

impl BoxTreeRoot {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        viewport: euclid::Size2D<f32, CSSPixel>,
    ) -> FragmentTreeRoot {
        let style = ComputedValues::initial_values();

        // FIXME: use the document’s mode:
        // https://drafts.csswg.org/css-writing-modes/#principal-flow
        let physical_containing_block = PhysicalRect::new(
            PhysicalPoint::zero(),
            PhysicalSize::new(Length::new(viewport.width), Length::new(viewport.height)),
        );
        let initial_containing_block = DefiniteContainingBlock {
            size: Vec2 {
                inline: physical_containing_block.size.width,
                block: physical_containing_block.size.height,
            },
            style,
        };

        let dummy_tree_rank = 0;
        let mut positioning_context = PositioningContext::new_for_initial_containing_block();
        let mut independent_layout = self.0.layout(
            layout_context,
            &mut positioning_context,
            &(&initial_containing_block).into(),
            dummy_tree_rank,
        );

        positioning_context.layout_in_initial_containing_block(
            layout_context,
            &initial_containing_block,
            &mut independent_layout.fragments,
        );

        let scrollable_overflow =
            independent_layout
                .fragments
                .iter()
                .fold(PhysicalRect::zero(), |acc, child| {
                    let child_overflow = child.scrollable_overflow();

                    // https://drafts.csswg.org/css-overflow/#scrolling-direction
                    // We want to clip scrollable overflow on box-start and inline-start
                    // sides of the scroll container.
                    //
                    // FIXME(mrobinson, bug 25564): This should take into account writing
                    // mode.
                    let child_overflow = PhysicalRect::new(
                        euclid::Point2D::zero(),
                        euclid::Size2D::new(
                            child_overflow.size.width + child_overflow.origin.x,
                            child_overflow.size.height + child_overflow.origin.y,
                        ),
                    );
                    acc.union(&child_overflow)
                });

        FragmentTreeRoot {
            children: independent_layout.fragments,
            scrollable_overflow,
            initial_containing_block: physical_containing_block,
        }
    }
}

impl FragmentTreeRoot {
    pub fn build_display_list(&self, builder: &mut crate::display_list::DisplayListBuilder) {
        for fragment in &self.children {
            fragment.build_display_list(builder, &self.initial_containing_block)
        }
    }

    pub fn print(&self) {
        let mut print_tree = PrintTree::new("Fragment Tree".to_string());
        for fragment in &self.children {
            fragment.print(&mut print_tree);
        }
    }

    pub fn scrollable_overflow(&self) -> webrender_api::units::LayoutSize {
        webrender_api::units::LayoutSize::from_untyped(Size2D::new(
            self.scrollable_overflow.size.width.px(),
            self.scrollable_overflow.size.height.px(),
        ))
    }

    fn iterate_through_fragments<F>(&self, process_func: &mut F)
    where
        F: FnMut(&Fragment, &PhysicalRect<Length>) -> bool,
    {
        fn do_iteration<M>(
            fragment: &Fragment,
            containing_block: &PhysicalRect<Length>,
            process_func: &mut M,
        ) -> bool
        where
            M: FnMut(&Fragment, &PhysicalRect<Length>) -> bool,
        {
            if !process_func(fragment, containing_block) {
                return false;
            }

            match fragment {
                Fragment::Box(fragment) => {
                    let new_containing_block = fragment
                        .content_rect
                        .to_physical(fragment.style.writing_mode, containing_block)
                        .translate(containing_block.origin.to_vector());
                    for child in &fragment.children {
                        if !do_iteration(child, &new_containing_block, process_func) {
                            return false;
                        }
                    }
                },
                Fragment::Anonymous(fragment) => {
                    let new_containing_block = fragment
                        .rect
                        .to_physical(fragment.mode, containing_block)
                        .translate(containing_block.origin.to_vector());
                    for child in &fragment.children {
                        if !do_iteration(child, &new_containing_block, process_func) {
                            return false;
                        }
                    }
                },
                _ => {},
            }

            true
        }

        for child in &self.children {
            if !do_iteration(child, &self.initial_containing_block, process_func) {
                break;
            }
        }
    }

    pub fn get_content_box_for_node(&self, requested_node: OpaqueNode) -> Rect<Au> {
        let mut rect = PhysicalRect::zero();
        let mut process = |fragment: &Fragment, containing_block: &PhysicalRect<Length>| {
            let fragment_relative_rect = match fragment {
                Fragment::Box(fragment) if fragment.tag == requested_node => fragment
                    .border_rect()
                    .to_physical(fragment.style.writing_mode, &containing_block),
                Fragment::Text(fragment) if fragment.tag == requested_node => fragment
                    .rect
                    .to_physical(fragment.parent_style.writing_mode, &containing_block),
                Fragment::Box(_) |
                Fragment::Text(_) |
                Fragment::Image(_) |
                Fragment::Anonymous(_) => return true,
            };

            rect = fragment_relative_rect
                .translate(containing_block.origin.to_vector())
                .union(&rect);
            return true;
        };

        self.iterate_through_fragments(&mut process);

        Rect::new(
            Point2D::new(
                Au::from_f32_px(rect.origin.x.px()),
                Au::from_f32_px(rect.origin.y.px()),
            ),
            Size2D::new(
                Au::from_f32_px(rect.size.width.px()),
                Au::from_f32_px(rect.size.height.px()),
            ),
        )
    }

    pub fn get_border_dimensions_for_node(&self, requested_node: OpaqueNode) -> Rect<i32> {
        let mut border_dimensions = Rect::zero();
        let mut process = |fragment: &Fragment, containing_block: &PhysicalRect<Length>| {
            let (style, padding_rect) = match fragment {
                Fragment::Box(fragment) if fragment.tag == requested_node => {
                    (&fragment.style, fragment.padding_rect())
                },
                Fragment::Box(_) |
                Fragment::Text(_) |
                Fragment::Image(_) |
                Fragment::Anonymous(_) => return true,
            };

            // https://drafts.csswg.org/cssom-view/#dom-element-clienttop
            // " If the element has no associated CSS layout box or if the
            //   CSS layout box is inline, return zero." For this check we
            // also explicitly ignore the list item portion of the display
            // style.
            let display = &style.get_box().display;
            if display.inside() == style::values::specified::box_::DisplayInside::Flow &&
                display.outside() == style::values::specified::box_::DisplayOutside::Inline
            {
                return false;
            }

            let style_structs::Border {
                border_top_width: top_width,
                border_left_width: left_width,
                ..
            } = *style.get_border();

            let padding_rect = padding_rect.to_physical(style.writing_mode, &containing_block);
            border_dimensions.size.width = padding_rect.size.width.px() as i32;
            border_dimensions.size.height = padding_rect.size.height.px() as i32;
            border_dimensions.origin.y = top_width.px() as i32;
            border_dimensions.origin.x = left_width.px() as i32;

            false
        };

        self.iterate_through_fragments(&mut process);

        border_dimensions
    }
}
