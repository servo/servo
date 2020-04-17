/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::display_list::stacking_context::{
    ContainingBlock, ContainingBlockInfo, StackingContext, StackingContextBuildMode,
    StackingContextBuilder,
};
use crate::dom_traversal::{Contents, NodeExt};
use crate::element_data::LayoutBox;
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
use crate::style_ext::{Display, DisplayGeneratingBox};
use crate::DefiniteContainingBlock;
use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use gfx_traits::print_tree::PrintTree;
use script_layout_interface::wrapper_traits::LayoutNode;
use servo_arc::Arc;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style_traits::CSSPixel;

#[derive(Serialize)]
pub struct BoxTreeRoot(BlockFormattingContext);

#[derive(Serialize)]
pub struct FragmentTreeRoot {
    /// The children of the root of the fragment tree.
    children: Vec<ArcRefCell<Fragment>>,

    /// The scrollable overflow of the root of the fragment tree.
    scrollable_overflow: PhysicalRect<Length>,

    /// The containing block used in the layout of this fragment tree.
    initial_containing_block: PhysicalRect<Length>,
}

impl BoxTreeRoot {
    pub fn construct<'dom, Node>(context: &LayoutContext, root_element: Node) -> Self
    where
        Node: 'dom + Copy + LayoutNode<'dom> + Send + Sync,
    {
        let (contains_floats, boxes) = construct_for_root_element(&context, root_element);

        // Zero box for `:root { display: none }`, one for the root element otherwise.
        assert!(boxes.len() <= 1);

        Self(BlockFormattingContext {
            contains_floats: contains_floats == ContainsFloats::Yes,
            contents: BlockContainer::BlockLevelBoxes(boxes),
        })
    }
}

fn construct_for_root_element<'dom>(
    context: &LayoutContext,
    root_element: impl NodeExt<'dom>,
) -> (ContainsFloats, Vec<ArcRefCell<BlockLevelBox>>) {
    let style = root_element.style(context);
    let box_style = style.get_box();

    let display_inside = match Display::from(box_style.display) {
        Display::None => {
            root_element.unset_boxes_in_subtree();
            return (ContainsFloats::No, Vec::new());
        },
        Display::Contents => {
            // Unreachable because the style crate adjusts the computed values:
            // https://drafts.csswg.org/css-display-3/#transformations
            // “'display' of 'contents' computes to 'block' on the root element”
            unreachable!()
        },
        // The root element is blockified, ignore DisplayOutside
        Display::GeneratingBox(DisplayGeneratingBox::OutsideInside { inside, .. }) => inside,
    };

    let contents =
        ReplacedContent::for_element(root_element).map_or(Contents::OfElement, Contents::Replaced);
    let (contains_floats, root_box) = if box_style.position.is_absolutely_positioned() {
        (
            ContainsFloats::No,
            BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(Arc::new(
                AbsolutelyPositionedBox::construct(
                    context,
                    root_element,
                    style,
                    display_inside,
                    contents,
                ),
            )),
        )
    } else if box_style.float.is_floating() {
        (
            ContainsFloats::Yes,
            BlockLevelBox::OutOfFlowFloatBox(FloatBox::construct(
                context,
                root_element,
                style,
                display_inside,
                contents,
            )),
        )
    } else {
        let propagated_text_decoration_line = style.clone_text_decoration_line();
        (
            ContainsFloats::No,
            BlockLevelBox::Independent(IndependentFormattingContext::construct(
                context,
                root_element,
                style,
                display_inside,
                contents,
                ContentSizesRequest::None,
                propagated_text_decoration_line,
            )),
        )
    };
    let root_box = ArcRefCell::new(root_box);
    root_element
        .element_box_slot()
        .set(LayoutBox::BlockLevel(root_box.clone()));
    (contains_floats, vec![root_box])
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
        let mut positioning_context =
            PositioningContext::new_for_containing_block_for_all_descendants();
        let independent_layout = self.0.layout(
            layout_context,
            &mut positioning_context,
            &(&initial_containing_block).into(),
            dummy_tree_rank,
        );

        let mut children = independent_layout
            .fragments
            .into_iter()
            .map(|fragment| ArcRefCell::new(fragment))
            .collect::<Vec<_>>();

        // Zero box for `:root { display: none }`, one for the root element otherwise.
        assert!(children.len() <= 1);

        // There may be more fragments at the top-level
        // (for positioned boxes whose containing is the initial containing block)
        // but only if there was one fragment for the root element.
        positioning_context.layout_initial_containing_block_children(
            layout_context,
            &initial_containing_block,
            &mut children,
        );

        let scrollable_overflow = children.iter().fold(PhysicalRect::zero(), |acc, child| {
            let child_overflow = child
                .borrow()
                .scrollable_overflow(&physical_containing_block);

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
            children,
            scrollable_overflow,
            initial_containing_block: physical_containing_block,
        }
    }
}

impl FragmentTreeRoot {
    pub fn build_display_list(&self, builder: &mut crate::display_list::DisplayListBuilder) {
        let mut stacking_context = StackingContext::create_root(&builder.wr);
        {
            let mut stacking_context_builder = StackingContextBuilder::new(&mut builder.wr);
            let containing_block_info = ContainingBlockInfo {
                rect: self.initial_containing_block,
                nearest_containing_block: None,
                containing_block_for_all_descendants: ContainingBlock::new(
                    &self.initial_containing_block,
                    stacking_context_builder.current_space_and_clip,
                ),
            };

            for fragment in &self.children {
                fragment.borrow().build_stacking_context_tree(
                    fragment,
                    &mut stacking_context_builder,
                    &containing_block_info,
                    &mut stacking_context,
                    StackingContextBuildMode::SkipHoisted,
                );
            }
        }

        stacking_context.sort();
        stacking_context.build_display_list(builder);
    }

    pub fn print(&self) {
        let mut print_tree = PrintTree::new("Fragment Tree".to_string());
        for fragment in &self.children {
            fragment.borrow().print(&mut print_tree);
        }
    }

    pub fn scrollable_overflow(&self) -> webrender_api::units::LayoutSize {
        webrender_api::units::LayoutSize::from_untyped(Size2D::new(
            self.scrollable_overflow.size.width.px(),
            self.scrollable_overflow.size.height.px(),
        ))
    }

    pub(crate) fn find<T>(
        &self,
        mut process_func: impl FnMut(&Fragment, &PhysicalRect<Length>) -> Option<T>,
    ) -> Option<T> {
        self.children.iter().find_map(|child| {
            child
                .borrow()
                .find(&self.initial_containing_block, &mut process_func)
        })
    }

    pub fn get_content_box_for_node(&self, requested_node: OpaqueNode) -> Rect<Au> {
        let mut bounding_box = PhysicalRect::zero();
        self.find(|fragment, containing_block| {
            if fragment.tag() != Some(requested_node) {
                return None::<()>;
            }

            let fragment_relative_rect = match fragment {
                Fragment::Box(fragment) => fragment
                    .border_rect()
                    .to_physical(fragment.style.writing_mode, &containing_block),
                Fragment::Text(fragment) => fragment
                    .rect
                    .to_physical(fragment.parent_style.writing_mode, &containing_block),
                Fragment::AbsoluteOrFixedPositioned(_) |
                Fragment::Image(_) |
                Fragment::Anonymous(_) => return None,
            };

            bounding_box = fragment_relative_rect
                .translate(containing_block.origin.to_vector())
                .union(&bounding_box);
            None::<()>
        });

        Rect::new(
            Point2D::new(
                Au::from_f32_px(bounding_box.origin.x.px()),
                Au::from_f32_px(bounding_box.origin.y.px()),
            ),
            Size2D::new(
                Au::from_f32_px(bounding_box.size.width.px()),
                Au::from_f32_px(bounding_box.size.height.px()),
            ),
        )
    }

    pub fn get_border_dimensions_for_node(&self, requested_node: OpaqueNode) -> Rect<i32> {
        self.find(|fragment, containing_block| {
            let (style, padding_rect) = match fragment {
                Fragment::Box(fragment) if fragment.tag == requested_node => {
                    (&fragment.style, fragment.padding_rect())
                },
                Fragment::AbsoluteOrFixedPositioned(_) |
                Fragment::Box(_) |
                Fragment::Text(_) |
                Fragment::Image(_) |
                Fragment::Anonymous(_) => return None,
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
                return Some(Rect::zero());
            }

            let padding_rect = padding_rect.to_physical(style.writing_mode, &containing_block);
            let border = style.get_border();
            Some(Rect::new(
                Point2D::new(
                    border.border_left_width.px() as i32,
                    border.border_top_width.px() as i32,
                ),
                Size2D::new(
                    padding_rect.size.width.px() as i32,
                    padding_rect.size.height.px() as i32,
                ),
            ))
        })
        .unwrap_or_else(Rect::zero)
    }
}
