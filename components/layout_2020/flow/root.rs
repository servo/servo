/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::display_list::IsContentful;
use crate::dom_traversal::{Contents, NodeExt};
use crate::flow::construct::ContainsFloats;
use crate::flow::float::FloatBox;
use crate::flow::{BlockContainer, BlockFormattingContext, BlockLevelBox};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragments::Fragment;
use crate::geom;
use crate::geom::flow_relative::Vec2;
use crate::positioned::AbsolutelyPositionedBox;
use crate::replaced::ReplacedContent;
use crate::style_ext::{
    Direction, Display, DisplayGeneratingBox, DisplayInside, DisplayOutside, WritingMode,
};
use crate::{ContainingBlock, DefiniteContainingBlock};
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use script_layout_interface::wrapper_traits::LayoutNode;
use servo_arc::Arc;
use style::context::SharedStyleContext;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};
use style::Zero;
use style_traits::CSSPixel;

pub struct BoxTreeRoot(BlockFormattingContext);
pub struct FragmentTreeRoot(Vec<Fragment>);

impl BoxTreeRoot {
    pub fn construct<'dom, Node>(context: &SharedStyleContext<'_>, root_element: Node) -> Self
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
    context: &SharedStyleContext<'_>,
    root_element: impl NodeExt<'dom>,
) -> (ContainsFloats, Vec<Arc<BlockLevelBox>>) {
    let style = root_element.style(context);
    let replaced = ReplacedContent::for_element(root_element, context);
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

    if let Some(replaced) = replaced {
        let _box = match replaced {};
        #[allow(unreachable_code)]
        {
            return (ContainsFloats::No, vec![Arc::new(_box)]);
        }
    }

    let position = box_style.position;
    let float = box_style.float;
    let contents = IndependentFormattingContext::construct(
        context,
        style,
        display_inside,
        Contents::OfElement(root_element),
    );
    if position.is_absolutely_positioned() {
        (
            ContainsFloats::No,
            vec![Arc::new(BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(
                AbsolutelyPositionedBox { contents },
            ))],
        )
    } else if float.is_floating() {
        (
            ContainsFloats::Yes,
            vec![Arc::new(BlockLevelBox::OutOfFlowFloatBox(FloatBox {
                contents,
            }))],
        )
    } else {
        (
            ContainsFloats::No,
            vec![Arc::new(BlockLevelBox::Independent(contents))],
        )
    }
}

impl BoxTreeRoot {
    pub fn layout(
        &self,
        layout_context: &LayoutContext,
        viewport: geom::Size<CSSPixel>,
    ) -> FragmentTreeRoot {
        let initial_containing_block_size = Vec2 {
            inline: Length::new(viewport.width),
            block: Length::new(viewport.height),
        };

        let initial_containing_block = ContainingBlock {
            inline_size: initial_containing_block_size.inline,
            block_size: LengthOrAuto::LengthPercentage(initial_containing_block_size.block),
            // FIXME: use the document’s mode:
            // https://drafts.csswg.org/css-writing-modes/#principal-flow
            mode: (WritingMode::HorizontalTb, Direction::Ltr),
        };
        let dummy_tree_rank = 0;
        let mut absolutely_positioned_fragments = vec![];
        let mut independent_layout = self.0.layout(
            layout_context,
            &initial_containing_block,
            dummy_tree_rank,
            &mut absolutely_positioned_fragments,
        );

        let initial_containing_block = DefiniteContainingBlock {
            size: initial_containing_block_size,
            mode: initial_containing_block.mode,
        };
        independent_layout.fragments.par_extend(
            absolutely_positioned_fragments
                .par_iter()
                .map(|a| a.layout(layout_context, &initial_containing_block)),
        );
        FragmentTreeRoot(independent_layout.fragments)
    }
}

impl FragmentTreeRoot {
    pub fn build_display_list(
        &self,
        builder: &mut crate::display_list::DisplayListBuilder,
        pipeline_id: msg::constellation_msg::PipelineId,
        viewport_size: webrender_api::units::LayoutSize,
    ) -> IsContentful {
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
        let mut is_contentful = IsContentful(false);
        for fragment in &self.0 {
            fragment.build_display_list(builder, &mut is_contentful, &containing_block)
        }
        is_contentful
    }
}
