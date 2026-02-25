/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use geom::{FlexAxis, MainStartCrossStart};
use malloc_size_of_derive::MallocSizeOf;
use script::layout_dom::ServoThreadSafeLayoutNode;
use servo_arc::Arc as ServoArc;
use style::context::SharedStyleContext;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::values::computed::ContentDistribution;
use style::values::specified::align::AlignFlags;

use crate::PropagatedBoxTreeData;
use crate::cell::ArcRefCell;
use crate::construct_modern::{ModernContainerBuilder, ModernItemKind};
use crate::context::LayoutContext;
use crate::dom::{LayoutBox, WeakLayoutBox};
use crate::dom_traversal::{NodeAndStyleInfo, NonReplacedContents};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::BaseFragmentInfo;
use crate::layout_box_base::LayoutBoxBase;
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
    align_content: ContentDistribution,
    align_items: AlignItems,
    justify_content: ContentDistribution,
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
                let flex_item_box = match item.kind {
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

                item.box_slot
                    .set(LayoutBox::FlexLevel(flex_item_box.clone()));
                flex_item_box
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

#[expect(clippy::large_enum_variant)]
#[derive(Debug, MallocSizeOf)]
pub(crate) enum FlexLevelBox {
    FlexItem(FlexItemBox),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
}

impl FlexLevelBox {
    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoThreadSafeLayoutNode,
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

    pub(crate) fn with_base<T>(&self, callback: impl FnOnce(&LayoutBoxBase) -> T) -> T {
        match self {
            FlexLevelBox::FlexItem(flex_item_box) => {
                callback(&flex_item_box.independent_formatting_context.base)
            },
            FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                callback(&positioned_box.borrow().context.base)
            },
        }
    }

    pub(crate) fn with_base_mut<T>(&mut self, callback: impl FnOnce(&mut LayoutBoxBase) -> T) -> T {
        match self {
            FlexLevelBox::FlexItem(flex_item_box) => {
                callback(&mut flex_item_box.independent_formatting_context.base)
            },
            FlexLevelBox::OutOfFlowAbsolutelyPositionedBox(positioned_box) => {
                callback(&mut positioned_box.borrow_mut().context.base)
            },
        }
    }

    pub(crate) fn attached_to_tree(&self, layout_box: WeakLayoutBox) {
        match self {
            Self::FlexItem(flex_item_box) => flex_item_box
                .independent_formatting_context
                .attached_to_tree(layout_box),
            Self::OutOfFlowAbsolutelyPositionedBox(positioned_box) => positioned_box
                .borrow_mut()
                .context
                .attached_to_tree(layout_box),
        }
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct FlexItemBox {
    pub(crate) independent_formatting_context: IndependentFormattingContext,
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

    pub(crate) fn style(&self) -> &ServoArc<ComputedValues> {
        self.independent_formatting_context.style()
    }

    fn base_fragment_info(&self) -> BaseFragmentInfo {
        self.independent_formatting_context.base_fragment_info()
    }
}
