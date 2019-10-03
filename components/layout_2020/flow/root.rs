/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom_traversal::{Contents, NodeExt};
use crate::flow::construct::ContainsFloats;
use crate::flow::float::FloatBox;
use crate::flow::{BlockContainer, BlockFormattingContext, BlockLevelBox};
use crate::fragments::Fragment;
use crate::geom;
use crate::geom::flow_relative::Vec2;
use crate::positioned::AbsolutelyPositionedBox;
use crate::replaced::ReplacedContent;
use crate::style_ext::{
    Direction, Display, DisplayGeneratingBox, DisplayInside, DisplayOutside, WritingMode,
};
use crate::{ContainingBlock, DefiniteContainingBlock, IndependentFormattingContext};
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use servo_arc::Arc;
use style::context::SharedStyleContext;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};
use style_traits::CSSPixel;

#[derive(Debug)]
pub struct BoxTreeRoot(BlockFormattingContext);

impl BoxTreeRoot {
    pub fn construct<'dom>(
        context: &SharedStyleContext<'_>,
        root_element: impl NodeExt<'dom>,
    ) -> Self {
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

    let contents = IndependentFormattingContext::construct(
        context,
        &style,
        display_inside,
        Contents::OfElement(root_element),
    );
    if box_style.position.is_absolutely_positioned() {
        (
            ContainsFloats::No,
            vec![Arc::new(BlockLevelBox::OutOfFlowAbsolutelyPositionedBox(
                AbsolutelyPositionedBox { style, contents },
            ))],
        )
    } else if box_style.float.is_floating() {
        (
            ContainsFloats::Yes,
            vec![Arc::new(BlockLevelBox::OutOfFlowFloatBox(FloatBox {
                contents,
                style,
            }))],
        )
    } else {
        (
            ContainsFloats::No,
            vec![Arc::new(BlockLevelBox::Independent { style, contents })],
        )
    }
}

impl BoxTreeRoot {
    fn layout(&self, viewport: geom::Size<CSSPixel>) -> Vec<Fragment> {
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
        let mut flow_children = self.0.layout(
            &initial_containing_block,
            dummy_tree_rank,
            &mut absolutely_positioned_fragments,
        );

        let initial_containing_block = DefiniteContainingBlock {
            size: initial_containing_block_size,
            mode: initial_containing_block.mode,
        };
        flow_children.fragments.par_extend(
            absolutely_positioned_fragments
                .par_iter()
                .map(|a| a.layout(&initial_containing_block)),
        );
        flow_children.fragments
    }
}
