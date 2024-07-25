/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;

use crate::cell::ArcRefCell;
use crate::formatting_contexts::IndependentFormattingContext;
use crate::positioned::AbsolutelyPositionedBox;

mod construct;
mod geom;
mod layout;

#[derive(Debug, Serialize)]
pub(crate) struct FlexContainer {
    children: Vec<ArcRefCell<FlexLevelBox>>,

    #[serde(skip_serializing)]
    style: ServoArc<ComputedValues>,
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
}
