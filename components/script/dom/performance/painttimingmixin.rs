/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use paint_api::display_list::PaintTimingInfo;
use servo_base::cross_process_instant::CrossProcessInstant;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct PaintTimingMixin {
    /// <https://www.w3.org/TR/paint-timing/#paint-timing-info>
    #[no_trace]
    paint_timing_info: PaintTimingInfo,
}

impl PaintTimingMixin {
    pub(crate) fn new_inherited(paint_timing_info: PaintTimingInfo) -> Self {
        Self { paint_timing_info }
    }

    pub(crate) fn presentation_time(&self) -> Option<CrossProcessInstant> {
        self.paint_timing_info.presentation_time
    }
}
