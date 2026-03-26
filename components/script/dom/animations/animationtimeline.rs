/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::num::Finite;
use script_bindings::reflector::Reflector;
use time::Duration;

use crate::dom::bindings::codegen::Bindings::AnimationTimelineBinding::AnimationTimelineMethods;

/// <https://drafts.csswg.org/web-animations-1/#animationtimeline>
#[dom_struct]
pub(crate) struct AnimationTimeline {
    reflector_: Reflector,
    /// <https://drafts.csswg.org/web-animations-1/#dom-animationtimeline-currenttime>
    ///
    /// The current time of this [`AnimationTimeline`] expressed as a [`time::Duration`] since the
    /// Document's "time origin." Note that this Duration may be negative.
    #[no_trace]
    current_time: Cell<Duration>,
}

impl AnimationTimeline {
    pub(crate) fn new_inherited(current_time: Duration) -> Self {
        Self {
            reflector_: Reflector::new(),
            current_time: Cell::new(current_time),
        }
    }

    pub(crate) fn current_time_in_seconds(&self) -> f64 {
        self.current_time.get().as_seconds_f64()
    }

    pub(crate) fn set_current_time(&self, duration: Duration) {
        self.current_time.set(duration);
    }

    pub(crate) fn advance_specific(&self, by: Duration) {
        self.current_time.set(self.current_time.get() + by);
    }
}

impl AnimationTimelineMethods<crate::DomTypeHolder> for AnimationTimeline {
    /// <https://drafts.csswg.org/web-animations-1/#dom-animationtimeline-currenttime>
    fn GetCurrentTime(&self) -> Option<Finite<f64>> {
        Finite::new(self.current_time.get().as_seconds_f64() * 1000.0)
    }
}
