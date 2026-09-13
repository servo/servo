/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_base::cross_process_instant::CrossProcessInstant;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct PaintTimingMixin {
    /// <https://w3c.github.io/paint-timing/#dom-painttimingmixin-presentationtime>
    #[no_trace]
    presentation_time: Option<CrossProcessInstant>,
}

impl PaintTimingMixin {
    pub(crate) fn new(presentation_time: Option<CrossProcessInstant>) -> Self {
        Self { presentation_time }
    }

    pub(crate) fn presentation_time(&self) -> Option<CrossProcessInstant> {
        self.presentation_time
    }
}
