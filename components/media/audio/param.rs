/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::block::{Block, FRAMES_PER_BLOCK_USIZE, Tick};
use crate::node::BlockInfo;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
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

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ParamDir {
    X,
    Y,
    Z,
}

/// An AudioParam.
///
/// <https://webaudio.github.io/web-audio-api/#AudioParam>
pub struct Param {
    val: f32,
    kind: ParamRate,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

        // move to next event if necessary
        // XXXManishearth k-rate events may get skipped over completely by this
        // method. Firefox currently doesn't support these, however, so we can
        // handle those later
        loop {
            let mut move_next = false;
            if let Some(done_time) = current_event.done_time() {
                // If this event is done, move on
                if done_time < current_tick {
                    move_next = true;
                }
            } else if let Some(next) = self.events.get(self.current_event + 1) {
                // this event has no done time and we must run it till the next one
                // starts
                if let Some(start_time) = next.start_time() {
                    // if the next one is ready to start, move on
                    if start_time <= current_tick {
                        move_next = true;
                    }
                } else {
                    // If we have a next event with no start time and
                    // the current event has no done time, this *has* to be because
                    // the current event is SetTargetAtTime and the next is a Ramp
                    // event. In this case we skip directly to the ramp assuming
                    // the SetTarget is ready to start (or has started already)
                    if current_event.time() <= current_tick {
                        move_next = true;
                    } else {
                        // This is a SetTarget event before its start time, ignore
                        return changed;
                    }
                }
            }
            if move_next {
                self.current_event += 1;
                self.event_start_value = self.val;
                self.event_start_time = current_tick;
                if let Some(next) = self.events.get(self.current_event + 1) {
                    current_event = next;
                    // may need to move multiple times
                    continue;
                } else {
                    return changed;
                }
            }
            break;
        }

        current_event.run(
            &mut self.val,
            current_tick,
            self.event_start_time,
            self.event_start_value,
        )
    }

    pub fn value(&self) -> f32 {
        // the data from connect()ed audionodes is first mixed
        // together in update(), and then mixed with the actual param value
        // https://webaudio.github.io/web-audio-api/#dom-audionode-connect-destinationparam-output
        self.val + self.block_mix_val
    }

    pub fn set_rate(&mut self, rate: ParamRate) {
        self.kind = rate;
    }

    pub(crate) fn insert_event(&mut self, event: AutomationEvent) {
        if let AutomationEvent::SetValue(val) = event {
            self.val = val;
            self.event_start_value = val;
            self.dirty = true;
            return;
        }

        let time = event.time();

        let result = self.events.binary_search_by(|e| e.time().cmp(&time));
        // XXXManishearth this should handle overlapping events
        let idx = match result {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        // XXXManishearth this isn't quite correct, this
        // doesn't handle cases for when this lands inside a running
        // event
        if let Some(is_hold) = event.cancel_event() {
            self.events.truncate(idx);
            if !is_hold {
                // If we cancelled the current event, reset
                // the value to what it was before
                if self.current_event >= self.events.len() {
                    self.val = self.event_start_value;
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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum RampKind {
    Linear,
    Exponential,
}

#[derive(Clone, PartialEq, Debug)]
/// <https://webaudio.github.io/web-audio-api/#dfn-automation-event>
pub(crate) enum AutomationEvent {
    SetValue(f32),
    SetValueAtTime(f32, Tick),
    RampToValueAtTime(RampKind, f32, Tick),
    SetTargetAtTime(f32, Tick, /* time constant, units of Tick */ f64),
    SetValueCurveAtTime(
        Vec<f32>,
        /* start time */ Tick,
        /* duration */ Tick,
    ),
    CancelAndHoldAtTime(Tick),
    CancelScheduledValues(Tick),
}

#[derive(Clone, PartialEq, Debug)]
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
                AutomationEvent::RampToValueAtTime(kind, val, Tick::from_time(time, rate))
            },
            UserAutomationEvent::SetValueCurveAtTime(values, start, duration) => {
                AutomationEvent::SetValueCurveAtTime(
                    values,
                    Tick::from_time(start, rate),
                    Tick::from_time(duration, rate),
                )
            },
            UserAutomationEvent::SetTargetAtTime(val, start, tau) => {
                AutomationEvent::SetTargetAtTime(
                    val,
                    Tick::from_time(start, rate),
                    tau * rate as f64,
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
            AutomationEvent::SetValueCurveAtTime(_, start, _) => start,
            AutomationEvent::RampToValueAtTime(_, _, tick) => tick,
            AutomationEvent::SetTargetAtTime(_, start, _) => start,
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
            AutomationEvent::RampToValueAtTime(_, _, tick) => Some(tick),
            AutomationEvent::SetValueCurveAtTime(_, start, duration) => Some(start + duration),
            AutomationEvent::SetTargetAtTime(..) => None,
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
            AutomationEvent::SetValueCurveAtTime(_, start, _) => Some(start),
            AutomationEvent::SetTargetAtTime(_, start, _) => Some(start),
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

    /// Update a parameter based on this event
    ///
    /// Returns true if something changed
    pub fn run(
        &self,
        value: &mut f32,
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
                    true
                } else {
                    false
                }
            },
            AutomationEvent::RampToValueAtTime(kind, val, time) => {
                let progress =
                    (current_tick - event_start_time).0 as f32 / (time - event_start_time).0 as f32;
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
                true
            },
            AutomationEvent::SetTargetAtTime(val, start, tau) => {
                let exp = -((current_tick - start) / tau);
                *value = val + (event_start_value - val) * exp.exp() as f32;
                true
            },
            AutomationEvent::SetValueCurveAtTime(ref values, start, duration) => {
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
                true
            },
            AutomationEvent::CancelAndHoldAtTime(..) => false,
            AutomationEvent::CancelScheduledValues(..) | AutomationEvent::SetValue(..) => {
                unreachable!("CancelScheduledValues/SetValue should never appear in the timeline")
            },
        }
    }
}
