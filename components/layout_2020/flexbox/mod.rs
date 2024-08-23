/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use geom::{FlexAxis, MainStartCrossStart};
use serde::Serialize;
use servo_arc::Arc as ServoArc;
use style::properties::longhands::align_items::computed_value::T as AlignItems;
use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;
use style::properties::ComputedValues;
use style::values::computed::{AlignContent, JustifyContent};
use style::values::specified::align::AlignFlags;

use crate::cell::ArcRefCell;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::BaseFragmentInfo;
use crate::positioned::AbsolutelyPositionedBox;

mod construct;
mod geom;
mod layout;

/// A structure to hold the configuration of a flex container for use during layout
/// and preferred width calculation.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct FlexContainerConfig {
    container_is_single_line: bool,
    flex_axis: FlexAxis,
    flex_direction: FlexDirection,
    flex_direction_is_reversed: bool,
    flex_wrap: FlexWrap,
    flex_wrap_reverse: bool,
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
            flex_axis,
            flex_direction,
            flex_direction_is_reversed,
            flex_wrap,
            flex_wrap_reverse,
            main_start_cross_start_sides_are,
            align_content,
            align_items,
            justify_content,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct FlexContainer {
    children: Vec<ArcRefCell<FlexLevelBox>>,

    #[serde(skip_serializing)]
    style: ServoArc<ComputedValues>,

    /// The configuration of this [`FlexContainer`].
    config: FlexContainerConfig,
}

impl FlexContainer {
    pub(crate) fn new(
        style: &ServoArc<ComputedValues>,
        children: Vec<ArcRefCell<FlexLevelBox>>,
    ) -> Self {
        Self {
            children,
            style: style.clone(),
            config: FlexContainerConfig::new(style),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum FlexLevelBox {
    FlexItem(FlexItemBox),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
}

#[derive(Debug, Serialize)]
pub(crate) struct FlexItemBox {
    independent_formatting_context: IndependentFormattingContext,
}

impl FlexItemBox {
    fn style(&self) -> &ServoArc<ComputedValues> {
        self.independent_formatting_context.style()
    }

    fn base_fragment_info(&self) -> BaseFragmentInfo {
        self.independent_formatting_context.base_fragment_info()
    }
}
