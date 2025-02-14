/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use geom::{FlexAxis, MainStartCrossStart};
use servo_arc::Arc as ServoArc;
use style::logical_geometry::WritingMode;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::properties::ComputedValues;
use style::values::computed::{AlignContent, JustifyContent};
use style::values::specified::align::AlignFlags;

use crate::cell::ArcRefCell;
use crate::construct_modern::{ModernContainerBuilder, ModernItemKind};
use crate::context::LayoutContext;
use crate::dom::{LayoutBox, NodeExt};
use crate::dom_traversal::{NodeAndStyleInfo, NonReplacedContents};
use crate::formatting_contexts::{IndependentFormattingContext, IndependentLayout};
use crate::fragment_tree::BaseFragmentInfo;
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};
use crate::{ContainingBlock, PropagatedBoxTreeData};

mod geom;
mod layout;

/// A structure to hold the configuration of a flex container for use during layout
/// and preferred width calculation.
#[derive(Clone, Debug)]
pub(crate) struct FlexContainerConfig {
    container_is_single_line: bool,
    writing_mode: WritingMode,
    flex_axis: FlexAxis,
    flex_direction: FlexDirection,
    flex_direction_is_reversed: bool,
    flex_wrap: FlexWrap,
    flex_wrap_is_reversed: bool,
    main_start_cross_start_sides_are: MainStartCrossStart,
    align_content: AlignContent,
    align_items: AlignItems,
    justify_content: JustifyContent,
}

impl FlexContainerConfig {
    fn new(container_style: &ComputedValues) -> FlexContainerConfig {
        let flex_direction = container_style.clone_flex_direction();
        let flex_axis = FlexAxis::from(flex_direction);
        let flex_wrap = container_style.get_position().flex_wrap;
        let container_is_single_line = match flex_wrap {
            FlexWrap::Nowrap => true,
            FlexWrap::Wrap | FlexWrap::WrapReverse => false,
        };
        let flex_direction_is_reversed = match flex_direction {
            FlexDirection::Row | FlexDirection::Column => false,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse => true,
        };
        let flex_wrap_reverse = match flex_wrap {
            FlexWrap::Nowrap | FlexWrap::Wrap => false,
            FlexWrap::WrapReverse => true,
        };

        let align_content = container_style.clone_align_content();
        let align_items = AlignItems(match container_style.clone_align_items().0 {
            AlignFlags::AUTO | AlignFlags::NORMAL => AlignFlags::STRETCH,
            align => align,
        });
        let justify_content = container_style.clone_justify_content();
        let main_start_cross_start_sides_are =
            MainStartCrossStart::from(flex_direction, flex_wrap_reverse);

        FlexContainerConfig {
            container_is_single_line,
            writing_mode: container_style.writing_mode,
            flex_axis,
            flex_direction,
            flex_direction_is_reversed,
            flex_wrap,
            flex_wrap_is_reversed: flex_wrap_reverse,
            main_start_cross_start_sides_are,
            align_content,
            align_items,
            justify_content,
        }
    }
}

#[derive(Debug)]
pub(crate) struct FlexContainer {
    children: Vec<ArcRefCell<FlexLevelBox>>,

    style: ServoArc<ComputedValues>,

    /// The configuration of this [`FlexContainer`].
    config: FlexContainerConfig,
}

impl FlexContainer {
    pub fn construct<'dom>(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<impl NodeExt<'dom>>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        let mut builder =
            ModernContainerBuilder::new(context, info, propagated_data.union(&info.style));
        contents.traverse(context, info, &mut builder);
        let items = builder.finish();

        let children = items
            .into_iter()
            .map(|item| {
                let box_ = match item.kind {
                    ModernItemKind::InFlow => ArcRefCell::new(FlexLevelBox::FlexItem(
                        FlexItemBox::new(item.formatting_context),
                    )),
                    ModernItemKind::OutOfFlow => {
                        let abs_pos_box =
                            ArcRefCell::new(AbsolutelyPositionedBox::new(item.formatting_context));
                        ArcRefCell::new(FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(abs_pos_box))
                    },
                };

                if let Some(box_slot) = item.box_slot {
                    box_slot.set(LayoutBox::FlexLevel(box_.clone()));
                }

                box_
            })
            .collect();

        Self {
            children,
            style: info.style.clone(),
            config: FlexContainerConfig::new(&info.style),
        }
    }
}

#[derive(Debug)]
pub(crate) enum FlexLevelBox {
    FlexItem(FlexItemBox),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
}

pub(crate) struct FlexItemBox {
    independent_formatting_context: IndependentFormattingContext,
    block_content_size_cache: ArcRefCell<Option<CachedBlockSizeContribution>>,
}

impl std::fmt::Debug for FlexItemBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FlexItemBox")
    }
}

impl FlexItemBox {
    fn new(independent_formatting_context: IndependentFormattingContext) -> Self {
        Self {
            independent_formatting_context,
            block_content_size_cache: Default::default(),
        }
    }

    fn style(&self) -> &ServoArc<ComputedValues> {
        self.independent_formatting_context.style()
    }

    fn base_fragment_info(&self) -> BaseFragmentInfo {
        self.independent_formatting_context.base_fragment_info()
    }
}

struct CachedBlockSizeContribution {
    containing_block_inline_size: Au,
    layout: IndependentLayout,
    positioning_context: PositioningContext,
}

impl CachedBlockSizeContribution {
    fn compatible_with_item_as_containing_block(
        &self,
        item_as_containing_block: &ContainingBlock,
    ) -> bool {
        item_as_containing_block.size.inline == self.containing_block_inline_size &&
            !item_as_containing_block.size.block.is_definite()
    }
}
