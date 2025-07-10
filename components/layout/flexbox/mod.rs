/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use geom::{FlexAxis, MainStartCrossStart};
use malloc_size_of_derive::MallocSizeOf;
use script::layout_dom::ServoLayoutNode;
use servo_arc::Arc as ServoArc;
use style::context::SharedStyleContext;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::values::computed::{AlignContent, JustifyContent};
use style::values::specified::align::AlignFlags;

use crate::PropagatedBoxTreeData;
use crate::cell::ArcRefCell;
use crate::construct_modern::{ModernContainerBuilder, ModernItemKind};
use crate::context::LayoutContext;
use crate::dom::LayoutBox;
use crate::dom_traversal::{NodeAndStyleInfo, NonReplacedContents};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::{BaseFragmentInfo, Fragment};
use crate::positioned::AbsolutelyPositionedBox;

mod geom;
mod layout;

/// A structure to hold the configuration of a flex container for use during layout
/// and preferred width calculation.
#[derive(Clone, Debug, MallocSizeOf)]
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

#[derive(Debug, MallocSizeOf)]
pub(crate) struct FlexContainer {
    children: Vec<ArcRefCell<FlexLevelBox>>,

    style: ServoArc<ComputedValues>,

    /// The configuration of this [`FlexContainer`].
    config: FlexContainerConfig,
}

impl FlexContainer {
    pub fn construct(
        context: &LayoutContext,
        info: &NodeAndStyleInfo<'_>,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        let mut builder = ModernContainerBuilder::new(context, info, propagated_data);
        contents.traverse(context, info, &mut builder);
        let items = builder.finish();

        let children = items
            .into_iter()
            .map(|item| {
                let box_ = match item.kind {
                    ModernItemKind::InFlow(independent_formatting_context) => ArcRefCell::new(
                        FlexLevelBox::FlexItem(FlexItemBox::new(independent_formatting_context)),
                    ),
                    ModernItemKind::OutOfFlow(independent_formatting_context) => {
                        let abs_pos_box = ArcRefCell::new(AbsolutelyPositionedBox::new(
                            independent_formatting_context,
                        ));
                        ArcRefCell::new(FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(abs_pos_box))
                    },
                    ModernItemKind::ReusedBox(layout_box) => match layout_box {
                        LayoutBox::FlexLevel(flex_level_box) => flex_level_box,
                        _ => unreachable!(
                            "Undamaged flex level element should be associated with flex level box"
                        ),
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

    pub(crate) fn repair_style(&mut self, new_style: &ServoArc<ComputedValues>) {
        self.config = FlexContainerConfig::new(new_style);
        self.style = new_style.clone();
    }
}

#[derive(Debug, MallocSizeOf)]
pub(crate) enum FlexLevelBox {
    FlexItem(FlexItemBox),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
}

impl FlexLevelBox {
    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoLayoutNode,
        new_style: &ServoArc<ComputedValues>,
    ) {
        match self {
            FlexLevelBox::FlexItem(flex_item_box) => flex_item_box
                .independent_formatting_context
                .repair_style(context, node, new_style),
            FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => positioned_box
                .borrow_mut()
                .context
                .repair_style(context, node, new_style),
        }
    }

    pub(crate) fn clear_fragment_layout_cache(&self) {
        match self {
            FlexLevelBox::FlexItem(flex_item_box) => flex_item_box
                .independent_formatting_context
                .base
                .clear_fragment_layout_cache(),
            FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => positioned_box
                .borrow()
                .context
                .base
                .clear_fragment_layout_cache(),
        }
    }

    pub(crate) fn fragments(&self) -> Vec<Fragment> {
        match self {
            FlexLevelBox::FlexItem(flex_item_box) => flex_item_box
                .independent_formatting_context
                .base
                .fragments(),
            FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                positioned_box.borrow().context.base.fragments()
            },
        }
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct FlexItemBox {
    independent_formatting_context: IndependentFormattingContext,
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
        }
    }

    fn style(&self) -> &ServoArc<ComputedValues> {
        self.independent_formatting_context.style()
    }

    fn base_fragment_info(&self) -> BaseFragmentInfo {
        self.independent_formatting_context.base_fragment_info()
    }
}
