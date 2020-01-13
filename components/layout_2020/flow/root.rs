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
use crate::positioned::AbsolutelyPositionedBox;
use crate::positioned::PositioningContext;
use crate::replaced::ReplacedContent;
use crate::sizing::ContentSizesRequest;
use crate::style_ext::{Display, DisplayGeneratingBox, DisplayInside};
use crate::DefiniteContainingBlock;
use gfx_traits::print_tree::PrintTree;
use script_layout_interface::wrapper_traits::LayoutNode;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::Zero;
use style_traits::CSSPixel;

pub struct BoxTreeRoot(BlockFormattingContext);
pub struct FragmentTreeRoot(Vec<Fragment>);

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

        FragmentTreeRoot(independent_layout.fragments)
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
        for fragment in &self.0 {
            fragment.build_display_list(builder, &containing_block)
        }
    }

    pub fn print(&self) {
        let mut print_tree = PrintTree::new("Fragment Tree".to_string());
        for fragment in &self.0 {
            fragment.print(&mut print_tree);
        }
    }
}
