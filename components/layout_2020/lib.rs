/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![deny(unsafe_code)]

#[macro_use]
extern crate serde;

use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};
use style::Zero;

pub mod context;
pub mod data;
mod dom_traversal;
mod element_data;
mod flow;
mod fragments;
mod geom;
mod opaque_node;
mod positioned;
pub mod query;
mod replaced;
mod style_ext;
pub mod traversal;
pub mod wrapper;

pub use flow::BoxTreeRoot;

use crate::dom_traversal::{Contents, NodeExt};
use crate::flow::{BlockFormattingContext, FlowChildren};
use crate::geom::flow_relative::Vec2;
use crate::positioned::AbsolutelyPositionedFragment;
use crate::replaced::ReplacedContent;
use crate::style_ext::{ComputedValuesExt, Direction, Position, WritingMode};
use servo_arc::Arc;
use std::convert::TryInto;
use style::context::SharedStyleContext;
use style::values::specified::box_::DisplayInside;

/// https://drafts.csswg.org/css-display/#independent-formatting-context
#[derive(Debug)]
enum IndependentFormattingContext {
    Flow(BlockFormattingContext),

    // Not called FC in specs, but behaves close enough
    Replaced(ReplacedContent),
    // Other layout modes go here
}

enum NonReplacedIFC<'a> {
    Flow(&'a BlockFormattingContext),
}

impl IndependentFormattingContext {
    fn construct<'dom, 'style>(
        context: &SharedStyleContext<'style>,
        style: &Arc<ComputedValues>,
        display_inside: DisplayInside,
        contents: Contents<impl NodeExt<'dom>>,
    ) -> Self {
        match contents.try_into() {
            Ok(non_replaced) => match display_inside {
                DisplayInside::Flow | DisplayInside::FlowRoot => {
                    IndependentFormattingContext::Flow(BlockFormattingContext::construct(
                        context,
                        style,
                        non_replaced,
                    ))
                },
                DisplayInside::None | DisplayInside::Contents => panic!(":("),
            },
            Err(replaced) => IndependentFormattingContext::Replaced(replaced),
        }
    }

    fn as_replaced(&self) -> Result<&ReplacedContent, NonReplacedIFC> {
        match self {
            IndependentFormattingContext::Replaced(r) => Ok(r),
            IndependentFormattingContext::Flow(f) => Err(NonReplacedIFC::Flow(f)),
        }
    }

    fn layout<'a>(
        &'a self,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> FlowChildren {
        match self.as_replaced() {
            Ok(replaced) => match *replaced {},
            Err(ifc) => ifc.layout(containing_block, tree_rank, absolutely_positioned_fragments),
        }
    }
}

impl<'a> NonReplacedIFC<'a> {
    fn layout(
        &self,
        containing_block: &ContainingBlock,
        tree_rank: usize,
        absolutely_positioned_fragments: &mut Vec<AbsolutelyPositionedFragment<'a>>,
    ) -> FlowChildren {
        match self {
            NonReplacedIFC::Flow(bfc) => {
                bfc.layout(containing_block, tree_rank, absolutely_positioned_fragments)
            },
        }
    }
}

struct ContainingBlock {
    inline_size: Length,
    block_size: LengthOrAuto,
    mode: (WritingMode, Direction),
}

struct DefiniteContainingBlock {
    size: Vec2<Length>,
    mode: (WritingMode, Direction),
}

/// https://drafts.csswg.org/css2/visuren.html#relative-positioning
fn relative_adjustement(
    style: &ComputedValues,
    inline_size: Length,
    block_size: LengthOrAuto,
) -> Vec2<Length> {
    if style.get_box().position != Position::Relative {
        return Vec2::zero();
    }
    fn adjust(start: LengthOrAuto, end: LengthOrAuto) -> Length {
        match (start, end) {
            (LengthOrAuto::Auto, LengthOrAuto::Auto) => Length::zero(),
            (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(end)) => -end,
            (LengthOrAuto::LengthPercentage(start), _) => start,
        }
    }
    let block_size = block_size.auto_is(Length::zero);
    let box_offsets = style.box_offsets().map_inline_and_block_axes(
        |v| v.percentage_relative_to(inline_size),
        |v| v.percentage_relative_to(block_size),
    );
    Vec2 {
        inline: adjust(box_offsets.inline_start, box_offsets.inline_end),
        block: adjust(box_offsets.block_start, box_offsets.block_end),
    }
}

// FIXME: use std::mem::take when it’s stable.
// https://github.com/rust-lang/rust/issues/61129
fn take<T>(x: &mut T) -> T
where
    T: Default,
{
    std::mem::replace(x, Default::default())
}
