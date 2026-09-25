/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use malloc_size_of_derive::MallocSizeOf;

use crate::audio_node::BlockInfo;
use crate::block::{Block, FRAMES_PER_BLOCK_USIZE, Tick};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, MallocSizeOf)]
pub enum ParamType {
    Frequency,
    Detune,
    Gain,
    Q,
    Pan,
    PlaybackRate,
    Position(ParamDir),
    Forward(ParamDir),
    Up(ParamDir),
    Orientation(ParamDir),
    Offset,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, MallocSizeOf)]
pub enum ParamDir {
    X,
    Y,
    Z,
}

/// An AudioParam.
///
/// <https://webaudio.github.io/web-audio-api/#AudioParam>
#[derive(Debug)]
pub struct Param {
    val: f32,
    /// Time of the value update
    time: Tick,
    default_value: f32,
    val_range: (f32, f32),
    kind: ParamRate,
    /// AutomationEvent and Cancel Hold Tick, if applicable
    events: Vec<AutomationEvent>,
    current_event: usize,
    event_start_time: Tick,
    event_start_value: f32,
    /// Cache of inputs from connect()ed nodes
    blocks: Vec<Block>,
    /// The value of all connect()ed inputs mixed together, for this frame
    block_mix_val: f32,
    /// If true, `blocks` has been summed together into a single block
    summed: bool,
    dirty: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, MallocSizeOf)]
pub enum ParamRate {
    /// Value is held for entire block
    KRate,
    /// Value is updated each frame
    ARate,
}

impl Param {
    pub fn new(val: f32) -> Self {
        Param {
            val,
            time: Tick(0),
            default_value: val,
            val_range: (f32::MIN, f32::MAX),
            time: Tick(0),
            kind: ParamRate::ARate,
            events: vec![],
            current_event: 0,
            event_start_time: Tick(0),
            event_start_value: val,
            blocks: Vec::new(),
            block_mix_val: 0.,
            summed: false,
            dirty: false,
        }
    }

    pub fn new_krate(val: f32) -> Self {
        Param {
            val,
            time: Tick(0),
            default_value: val,
            val_range: (f32::MIN, f32::MAX),
            kind: ParamRate::KRate,
            events: vec![],
            current_event: 0,
            event_start_time: Tick(0),
            event_start_value: val,
            blocks: Vec::new(),
            block_mix_val: 0.,
            summed: false,
            dirty: false,
        }
    }

    fn get_previous_event(&self, index: usize) -> Option<&AutomationEvent> {
        if index > 0 {
            self.events.get(index - 1)
        } else {
            None
        }
    }

    fn get_previous_event_mut(&mut self, index: usize) -> Option<&mut AutomationEvent> {
        if index > 0 {
            self.events.get_mut(index - 1)
        } else {
            None
        }
    }

    /// Update the value of this param to the next
    ///
    /// Invariant: This should be called with monotonically increasing
    /// ticks, and Tick(0) should never be skipped.
    ///
    /// Returns true if anything changed
    pub fn update(&mut self, block: &BlockInfo, tick: Tick) -> bool {
        let mut changed = self.dirty;
        self.dirty = false;
        if tick.0 == 0 {
            self.summed = true;
            if let Some(first) = self.blocks.pop() {
                // first sum them together
                // https://webaudio.github.io/web-audio-api/#dom-audionode-connect-destinationparam-output
                let block = self
                    .blocks
                    .drain(..)
                    .fold(first, |acc, block| acc.sum(block));
                self.blocks.push(block);
            }
        } else if self.kind == ParamRate::KRate {
            return changed;
        }

        // Even if the timeline does nothing, it's still possible
        // that there were connected inputs, so we should not
        // directly return `false` after this point, instead returning
        // `changed`
        changed |= if let Some(block) = self.blocks.first() {
            // store to be summed with `val` later
            self.block_mix_val = block.data_chan_frame(tick.0 as usize, 0);
            true
        } else {
            false
        };

        if self.events.len() <= self.current_event {
            return changed;
        }

        let current_tick = block.absolute_tick(tick);
        let mut current_event = &self.events[self.current_event];

        // Update then move to next event if necessary.
        // This will also handle cases where we have overlap,
        // ie. two ramps with the same end time.
        loop {
            let maybe_next_event = self.events.get(self.current_event + 1);
            // Run the event automation if it is active.
            if current_event.is_active(
                current_tick,
                self.get_previous_event(self.current_event),
                maybe_next_event,
            ) {
                if let Some(start_time) = current_event.start_time() {
                    // If the current event just started, update the param event start time and val
                    if start_time == current_tick {
                        self.event_start_time = self.time;
                        self.event_start_value = self.val;
                        // > If the preceding event is a SetTarget event,
                        // > T0 and V0 are chosen from the current time and value of SetTarget automation.
                        // > That is, if the SetTarget event has not started, T0 is the start time of the event,
                        // > and V0 is the value just before the SetTarget event starts.
                        // > In this case, the LinearRampToValue event effectively replaces the SetTarget event.
                        if current_event.is_set_target() &&
                            maybe_next_event
                                .map(|next_event| next_event.is_ramp())
                                .unwrap_or_default()
                        {
                            continue;
                        }
                    }
                }
                // Update the value
                if current_event.run(
                    &mut self.val,
                    &mut self.time,
                    current_tick,
                    self.event_start_time,
                    self.event_start_value,
                ) {
                    // If at least one event updated the param value, then changed is true.
                    changed = true;
                }
            }
            // Continue to the next event if the next event is active.
            if let Some(next_event) = maybe_next_event &&
                next_event.is_active(
                    current_tick,
                    Some(current_event),
                    self.events.get(self.current_event + 2),
                )
            {
                self.current_event += 1;
                // If the next event is a ramp, set the param event start value and time.
                // This is because ramp does not have a start time.
                if next_event.is_ramp() {
                    self.event_start_time = self.time;
                    self.event_start_value = self.val;
                }
                current_event = next_event;
                continue;
            }
            break;
        }
        changed
    }

    pub fn value(&self) -> f32 {
        // the data from connect()ed audionodes is first mixed
        // together in update(), and then mixed with the actual param value
        // https://webaudio.github.io/web-audio-api/#dom-audionode-connect-destinationparam-output
        // Clamp values when they are to be applied to the output, not during automation.
        // > If the sum is NaN, replace the sum with the defaultValue.
        // Specs do not mention replacing positive / negative infinity with defaultValue.
        // However, nan-param.html WPT test suggests this behavior is expected.
        // https://webaudio.github.io/web-audio-api/#computedvalue
        let mut computed_value = self.val + self.block_mix_val;
        if !computed_value.is_finite() {
            computed_value = self.default_value;
        }
        computed_value.clamp(self.val_range.0, self.val_range.1)
    }

    pub fn set_rate(&mut self, rate: ParamRate) {
        self.kind = rate;
    }

    pub(crate) fn update_range(&mut self, range: (f32, f32)) {
        self.val_range = range;
    }

    pub(crate) fn insert_event(&mut self, event: AutomationEvent) {
        if let AutomationEvent::SetValue(val) = event {
            self.val = val;
            self.event_start_value = val;
            self.dirty = true;
            return;
        }

        let time = event.time();

        // > If one of these events is added at a time where there is already one or more events,
        // > then it will be placed in the list after them, but before events whose times are after the event.
        // https://webaudio.github.io/web-audio-api/#dfn-automation-method
        let result = self.events.binary_search_by(|e| {
            if e.time() <= time {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        let idx = match result {
            // Through custom comparator, binary search will never find a match.
            // It will always find the largest index with time greater than or equal to the event.
            Ok(_) => unreachable!(),
            Err(idx) => idx,
        };

        if let Some(is_hold) = event.cancel_event() {
            let mut events_to_keep = idx;
            // https://webaudio.github.io/web-audio-api/#dom-audioparam-cancelandholdattime
            if is_hold {
                // > the automation value that would have happened at cancelTime
                // > is then propagated for all future time until other automation events are introduced.
                //
                // If we have a hold, set the cancel hold tick for the event.
                // When updating the param, once we reach the hold tick, we will hold the current value.
                if let Some(overlap_event) = self.events.get_mut(idx) &&
                    overlap_event.is_ramp()
                {
                    overlap_event.set_cancel_tick(event.time());
                    // Keep the RampToValueAtTime event.
                    events_to_keep = idx + 1;
                } else {
                    if let Some(overlap_event) = self.get_previous_event_mut(idx) &&
                        (overlap_event.is_set_target() ||
                            // SetValueCurve always has done time
                            (overlap_event.is_set_value_curve() &&
                                overlap_event
                                    .done_time()
                                    .unwrap() >=
                                    event.time()))
                    {
                        overlap_event.set_cancel_tick(event.time());
                    }
                }
                self.events.truncate(events_to_keep);
            } else {
                // If the event preceding the cancel is not setValueAtTime,
                // then it is an active event.
                // > Any active automations whose automation event time is less
                // > than cancelTime are also cancelled.
                // https://webaudio.github.io/web-audio-api/#dom-audioparam-cancelscheduledvalues
                let mut overlap_start_time = None;
                if let Some(overlap_event) = self.get_previous_event(idx) &&
                    (overlap_event.is_set_target() ||
                        (overlap_event.is_set_value_curve() &&
                            // SetValueCurve always has done time
                            overlap_event
                                .done_time()
                                .unwrap() >=
                                event.time()))
                {
                    events_to_keep = idx - 1;
                    overlap_start_time = Some(overlap_event.time());
                }
                self.events.truncate(events_to_keep);
                // If the current value was set after the overlap start time,
                // then we need to reset the value to before the active event started.
                // > cancellations may cause discontinuities because
                // > the original value (from before such automation) is restored immediately.
                if let Some(overlap_start_time) = overlap_start_time &&
                    self.time > overlap_start_time
                {
                    self.val = self.event_start_value;
                    self.time = self.event_start_time;
                }
            }
            // don't actually insert the event
            return;
        }
        self.events.insert(idx, event);
        // XXXManishearth handle inserting events with a time before that
        // of the current one
    }

    pub(crate) fn add_block(&mut self, block: Block) {
        debug_assert!(block.chan_count() == 1);
        // summed only becomes true during a node's process() call,
        // but add_block is called during graph traversal before processing,
        // so if summed is true that means we've moved on to the next block
        // and should clear our inputs
        if self.summed {
            self.blocks.clear();
        }
        self.blocks.push(block)
    }

    /// Flush an entire block of values into a buffer
    ///
    /// Only for use with AudioListener.
    ///
    /// Invariant: `block` must be a FRAMES_PER_BLOCK length array filled with silence
    pub(crate) fn flush_to_block(&mut self, info: &BlockInfo, block: &mut [f32]) {
        // common case
        if self.current_event >= self.events.len() && self.blocks.is_empty() {
            if self.val != 0. {
                for block_tick in &mut block[0..FRAMES_PER_BLOCK_USIZE] {
                    // ideally this can use some kind of vectorized memset()
                    *block_tick = self.val;
                }
            }
        // if the value is zero, our buffer is already zeroed
        } else {
            for block_tick in &mut block[0..FRAMES_PER_BLOCK_USIZE] {
                self.update(info, Tick(*block_tick as u64));
                *block_tick = self.val;
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, MallocSizeOf)]
pub enum RampKind {
    Linear,
    Exponential,
}

#[derive(Clone, PartialEq, Debug)]
/// <https://webaudio.github.io/web-audio-api/#dfn-automation-event>
pub(crate) enum AutomationEvent {
    SetValue(f32),
    SetValueAtTime(f32, Tick),
    RampToValueAtTime(
        RampKind,
        f32,
        Tick,
        /* Cancel and Hold Tick */ Option<Tick>,
    ),
    SetTargetAtTime(
        f32,
        Tick,
        /* time constant, units of Tick */
        f64,
        // Cancel and Hold Tick
        Option<Tick>,
    ),
    SetValueCurveAtTime(
        Vec<f32>,
        /* start time */ Tick,
        /* duration */ Tick,
        // Cancel and Hold Tick
        Option<Tick>,
    ),
    CancelAndHoldAtTime(Tick),
    CancelScheduledValues(Tick),
}

#[derive(Clone, PartialEq, Debug, MallocSizeOf)]
/// An AutomationEvent that uses times in s instead of Ticks
pub enum UserAutomationEvent {
    SetValue(f32),
    SetValueAtTime(f32, /* time */ f64),
    RampToValueAtTime(RampKind, f32, /* time */ f64),
    SetTargetAtTime(f32, f64, /* time constant, units of s */ f64),
    SetValueCurveAtTime(Vec<f32>, /* start time */ f64, /* duration */ f64),
    CancelAndHoldAtTime(f64),
    CancelScheduledValues(f64),
}

impl UserAutomationEvent {
    pub(crate) fn convert_to_event(self, rate: f32) -> AutomationEvent {
        match self {
            UserAutomationEvent::SetValue(val) => AutomationEvent::SetValue(val),
            UserAutomationEvent::SetValueAtTime(val, time) => {
                AutomationEvent::SetValueAtTime(val, Tick::from_time(time, rate))
            },
            UserAutomationEvent::RampToValueAtTime(kind, val, time) => {
                AutomationEvent::RampToValueAtTime(kind, val, Tick::from_time(time, rate), None)
            },
            UserAutomationEvent::SetValueCurveAtTime(values, start, duration) => {
                AutomationEvent::SetValueCurveAtTime(
                    values,
                    Tick::from_time(start, rate),
                    Tick::from_time(duration, rate),
                    None,
                )
            },
            UserAutomationEvent::SetTargetAtTime(val, start, tau) => {
                AutomationEvent::SetTargetAtTime(
                    val,
                    Tick::from_time(start, rate),
                    tau * rate as f64,
                    None,
                )
            },
            UserAutomationEvent::CancelScheduledValues(t) => {
                AutomationEvent::CancelScheduledValues(Tick::from_time(t, rate))
            },
            UserAutomationEvent::CancelAndHoldAtTime(t) => {
                AutomationEvent::CancelAndHoldAtTime(Tick::from_time(t, rate))
            },
        }
    }
}

impl AutomationEvent {
    /// The time of the event used for ordering
    pub fn time(&self) -> Tick {
        match *self {
            AutomationEvent::SetValueAtTime(_, tick) => tick,
            AutomationEvent::SetValueCurveAtTime(_, start, _, _) => start,
            AutomationEvent::RampToValueAtTime(_, _, tick, cancel_tick) => {
                cancel_tick.unwrap_or(tick)
            },
            AutomationEvent::SetTargetAtTime(_, start, _, _) => start,
            AutomationEvent::CancelAndHoldAtTime(t) => t,
            AutomationEvent::CancelScheduledValues(tick) => tick,
            AutomationEvent::SetValue(..) => {
                unreachable!("SetValue should never appear in the timeline")
            },
        }
    }

    pub fn done_time(&self) -> Option<Tick> {
        match *self {
            AutomationEvent::SetValueAtTime(_, tick) => Some(tick),
            AutomationEvent::RampToValueAtTime(_, _, tick, cancel_tick) => {
                Some(cancel_tick.unwrap_or(tick))
            },
            AutomationEvent::SetValueCurveAtTime(_, start, duration, cancel_tick) => {
                Some(cancel_tick.unwrap_or(start + duration))
            },
            AutomationEvent::SetTargetAtTime(_, _, _, cancel_tick) => cancel_tick,
            AutomationEvent::CancelAndHoldAtTime(t) => Some(t),
            AutomationEvent::CancelScheduledValues(..) | AutomationEvent::SetValue(..) => {
                unreachable!("CancelScheduledValues/SetValue should never appear in the timeline")
            },
        }
    }

    pub fn start_time(&self) -> Option<Tick> {
        match *self {
            AutomationEvent::SetValueAtTime(_, tick) => Some(tick),
            AutomationEvent::RampToValueAtTime(..) => None,
            AutomationEvent::SetValueCurveAtTime(_, start, _, _) => Some(start),
            AutomationEvent::SetTargetAtTime(_, start, _, _) => Some(start),
            AutomationEvent::CancelAndHoldAtTime(t) => Some(t),
            AutomationEvent::CancelScheduledValues(..) | AutomationEvent::SetValue(..) => {
                unreachable!("CancelScheduledValues/SetValue should never appear in the timeline")
            },
        }
    }

    /// Returns Some if it's a cancel event
    /// the boolean is if it's CancelAndHold
    pub fn cancel_event(&self) -> Option<bool> {
        match *self {
            AutomationEvent::CancelAndHoldAtTime(..) => Some(true),
            AutomationEvent::CancelScheduledValues(..) => Some(false),
            _ => None,
        }
    }

    fn is_ramp(&self) -> bool {
        matches!(*self, AutomationEvent::RampToValueAtTime(..))
    }

    fn is_set_target(&self) -> bool {
        matches!(*self, AutomationEvent::SetTargetAtTime(..))
    }

    fn is_set_value_curve(&self) -> bool {
        matches!(*self, AutomationEvent::SetValueCurveAtTime(..))
    }

    fn is_active(
        &self,
        current_tick: Tick,
        previous_event: Option<&AutomationEvent>,
        next_event: Option<&AutomationEvent>,
    ) -> bool {
        match self {
            AutomationEvent::SetValueAtTime(_, time) => *time == current_tick,
            // https://webaudio.github.io/web-audio-api/#dom-audioparam-linearramptovalueattime
            AutomationEvent::RampToValueAtTime(..) => {
                // Ramp always has a done time.
                let done_time = self.done_time().unwrap();
                let previous_event_check = previous_event.is_none_or(|event| {
                    // > If the preceding event is a SetTarget event, T0 and V0
                    // > are chosen from the current time and value of SetTarget automation.
                    // > If the SetTarget event has already started, T0 is the current context time,
                    // > and V0 is the current SetTarget automation value at time T0.
                    //
                    // If previous event is an active SetTarget, then this ramp will be active,
                    // because T0 is set to current time.
                    if event.is_set_target() {
                        event.is_active(current_tick, None, None)
                    } else {
                        // > The value during the time interval T0≤t<T1
                        // > (where T0 is the time of the previous event
                        // > and T1 is the endTime parameter passed into this method) will be calculated as:
                        // > v(t)=V0+(V1−V0)*(t−T0)/(T1−T0)
                        // > where V0 is the value at the time T0 and V1 is the value parameter passed into this method.
                        //
                        // This means the event is not active until T0 <= t.
                        event.time() <= current_tick
                    }
                });
                // > If there are no more events after this LinearRampToValue event then for t≥T1, v(t)=V1.
                //
                // This means if there are no active events after this, the event is active at T1.
                // The specs are not explicit about what happens if the next event starts at T1.
                // Based on behavior from Firefox and Chrome, v(t)=V1 for t=T1 even if next event
                // starts at T1.
                // Therefore we consider the event active at T1.
                done_time >= current_tick && previous_event_check
            },
            // > During the time interval: T0≤t, where T0 is the startTime parameter.
            // https://webaudio.github.io/web-audio-api/#dom-audioparam-settargetattime
            AutomationEvent::SetTargetAtTime(_, start_time, _, _) => {
                // SetTarget has a start time. Once current tick reaches the start time,
                // the event becomes active.
                let next_event_check = next_event.is_none_or(|event| {
                    // If next event is Ramp, it will become active once the SetTarget is active.
                    // We need to handle the edge case where the SetTarget event is already started.
                    // In this case, as mentioned above, the value as of current context time will
                    // be used by the ramp as V0.
                    // Therefore this should return true as we would always want to calculate the value,
                    // assuming the other checks pass.
                    event.is_ramp() ||
                        // > For all other events, the SetTarget event ends at the time of the next event.
                        // So event is active until next event starts
                        event.time() > current_tick
                });
                // https://webaudio.github.io/web-audio-api/#dom-audioparam-cancelandholdattime
                // Calculate based on automation, and then at t_c the value calculated will be held.
                let cancel_and_hold_check = self
                    .done_time()
                    .is_none_or(|done_time| done_time >= current_tick);

                *start_time <= current_tick && next_event_check && cancel_and_hold_check
            },
            // https://webaudio.github.io/web-audio-api/#dom-audioparam-setvaluecurveattime
            AutomationEvent::SetValueCurveAtTime(_, start_time, _, _) => {
                // > An implicit call to setValueAtTime() is made at time T0+TD with value V[N−1]
                // > so that following automations will start from the end of the setValueCurveAtTime() event.
                //
                // Therefore when t = T0+TD, it is active because we need to calculate V(t).
                *start_time <=
                    current_tick &&
                    // SetValueCurve always has an end time.
                    self.done_time()
                        .unwrap() >=
                        current_tick
            },
            _ => false,
        }
    }

    fn set_cancel_tick(&mut self, cancel_tick: Tick) {
        match self {
            AutomationEvent::RampToValueAtTime(_, _, _, maybe_tick) |
            AutomationEvent::SetTargetAtTime(_, _, _, maybe_tick) |
            AutomationEvent::SetValueCurveAtTime(_, _, _, maybe_tick) => {
                maybe_tick.replace(cancel_tick);
            },
            _ => {},
        }
    }

    /// Update a parameter based on this event
    ///
    /// Returns true if something changed
    pub fn run(
        &self,
        value: &mut f32,
        value_time: &mut Tick,
        current_tick: Tick,
        event_start_time: Tick,
        event_start_value: f32,
    ) -> bool {
        if matches!(self.start_time(), Some(start_time) if start_time > current_tick) {
            // The previous event finished and we advanced to this
            // event, but it's not started yet. Return early
            return false;
        }

        match *self {
            AutomationEvent::SetValueAtTime(val, time) => {
                if current_tick == time {
                    *value = val;
                    *value_time = time;
                    true
                } else {
                    false
                }
            },
            AutomationEvent::RampToValueAtTime(kind, val, time, _) => {
                if time == event_start_time {
                    *value = val;
                } else {
                    let progress = (current_tick - event_start_time).0 as f32 /
                        (time - event_start_time).0 as f32;
                    match kind {
                        RampKind::Linear => {
                            *value = event_start_value + (val - event_start_value) * progress;
                        },
                        RampKind::Exponential => {
                            let ratio = val / event_start_value;
                            if event_start_value == 0. || ratio < 0. {
                                if time == current_tick {
                                    *value = val;
                                } else {
                                    *value = event_start_value;
                                }
                            } else {
                                *value = event_start_value * (ratio).powf(progress);
                            }
                        },
                    }
                }
                *value_time = current_tick;
                true
            },
            AutomationEvent::SetTargetAtTime(val, start, tau, _) => {
                let exp = -((current_tick - start) / tau);
                *value = val + (event_start_value - val) * exp.exp() as f32;
                *value_time = current_tick;
                true
            },
            AutomationEvent::SetValueCurveAtTime(ref values, start, duration, _) => {
                let progress = ((current_tick.0 as f32) - (start.0 as f32)) / (duration.0 as f32);
                debug_assert!(progress >= 0.);
                let n = values.len() as f32;
                let k_float = (n - 1.) * progress;
                let k = k_float.floor();
                if (k + 1.) < n {
                    let progress = k_float - k;
                    *value =
                        values[k as usize] * (1. - progress) + values[(k + 1.) as usize] * progress;
                } else {
                    *value = values[(n - 1.) as usize];
                }
                *value_time = current_tick;
                true
            },
            AutomationEvent::CancelAndHoldAtTime(..) => false,
            AutomationEvent::CancelScheduledValues(..) | AutomationEvent::SetValue(..) => {
                unreachable!("CancelScheduledValues/SetValue should never appear in the timeline")
            },
        }
    }
}
