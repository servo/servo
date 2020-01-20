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
use crate::geom;
use crate::geom::flow_relative::Vec2;
use crate::geom::physical;
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
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::Zero;
use style_traits::CSSPixel;

pub struct BoxTreeRoot(BlockFormattingContext);

pub struct FragmentTreeRoot {
    /// The children of the root of the fragment tree.
    children: Vec<Fragment>,

    /// The scrollable overflow of the root of the fragment tree.
    scrollable_overflow: physical::Rect<Length>,

    /// The axis-aligned bounding box of the border box of all child fragments
    bounding_box_of_border_boxes: physical::Rect<Length>,
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
        viewport: geom::Size<CSSPixel>,
    ) -> FragmentTreeRoot {
        let style = ComputedValues::initial_values();
        let initial_containing_block = DefiniteContainingBlock {
            size: Vec2 {
                inline: Length::new(viewport.width),
                block: Length::new(viewport.height),
            },
            // FIXME: use the document’s mode:
            // https://drafts.csswg.org/css-writing-modes/#principal-flow
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

        // FIXME(mrobinson, bug 25564): We should be using the containing block
        // here to properly convert scrollable overflow to physical geometry.
        let scrollable_overflow =
            independent_layout
                .fragments
                .iter()
                .fold(physical::Rect::zero(), |acc, child| {
                    let child_overflow = child.scrollable_overflow();

                    // https://drafts.csswg.org/css-overflow/#scrolling-direction
                    // We want to clip scrollable overflow on box-start and inline-start
                    // sides of the scroll container.
                    //
                    // FIXME(mrobinson, bug 25564): This should take into account writing
                    // mode.
                    let child_overflow = physical::Rect {
                        top_left: physical::Vec2::zero(),
                        size: physical::Vec2 {
                            x: child_overflow.size.x + child_overflow.top_left.x,
                            y: child_overflow.size.y + child_overflow.top_left.y,
                        },
                    };
                    acc.axis_aligned_bounding_box(&child_overflow)
                });

        let containing_block = physical::Rect::zero();
        let bounding_box_of_border_boxes =
            independent_layout
                .fragments
                .iter()
                .fold(physical::Rect::zero(), |acc, child| {
                    acc.axis_aligned_bounding_box(&match child {
                        Fragment::Box(fragment) => fragment
                            .border_rect()
                            .to_physical(fragment.style.writing_mode, &containing_block),
                        Fragment::Anonymous(fragment) => {
                            fragment.rect.to_physical(fragment.mode, &containing_block)
                        },
                        Fragment::Text(fragment) => fragment
                            .rect
                            .to_physical(fragment.parent_style.writing_mode, &containing_block),
                        Fragment::Image(fragment) => fragment
                            .rect
                            .to_physical(fragment.style.writing_mode, &containing_block),
                    })
                });

        FragmentTreeRoot {
            children: independent_layout.fragments,
            scrollable_overflow,
            bounding_box_of_border_boxes,
        }
    }
}

impl FragmentTreeRoot {
    pub fn build_display_list(
        &self,
        builder: &mut crate::display_list::DisplayListBuilder,
        viewport_size: webrender_api::units::LayoutSize,
    ) {
        let containing_block = geom::physical::Rect {
            top_left: geom::physical::Vec2 {
                x: Length::zero(),
                y: Length::zero(),
            },
            size: geom::physical::Vec2 {
                x: Length::new(viewport_size.width),
                y: Length::new(viewport_size.height),
            },
        };
        for fragment in &self.children {
            fragment.build_display_list(builder, &containing_block)
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
            self.scrollable_overflow.size.x.px(),
            self.scrollable_overflow.size.y.px(),
        ))
    }

    pub fn bounding_box_of_border_boxes(&self) -> Rect<Au> {
        let origin = Point2D::new(
            Au::from_f32_px(self.bounding_box_of_border_boxes.top_left.x.px()),
            Au::from_f32_px(self.bounding_box_of_border_boxes.top_left.y.px()),
        );
        let size = Size2D::new(
            Au::from_f32_px(self.bounding_box_of_border_boxes.size.x.px()),
            Au::from_f32_px(self.bounding_box_of_border_boxes.size.y.px()),
        );
        Rect::new(origin, size)
    }
}
