/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Translation of platform scroll input into servo [WheelDelta]s.
//!
//! The coasting (momentum) that carries a touchpad gesture on after the fingers are lifted lives
//! in the second half of this module: it is fed the same axis values as the conversion above, and
//! mirrors them the same single way, so a gesture hands over to the animation without a jump.
//!
//! Embedders receive wheel and touchpad scrolling in the unit of their platform, while
//! [`servo::InputEvent::Wheel`] takes a [`WheelDelta`] in device pixels whose `y` is positive when
//! the viewport is scrolled up. Every source reports its values in the frame of the content, which is
//! the opposite of the frame of the viewport, so the arithmetic that bridges the two is the same for
//! all of them and is platform independent: it lives here, outside of the platform specific embedder
//! modules, and is unit tested on the host. Telling the two kinds of input apart is left to the
//! platform module, because the tool types are part of the platform API.
//!
//! OpenHarmony does not hand the system momentum to an `XComponent`, and ArkWeb does not take it
//! either: ArkWeb builds its own inertia in `ui::NativeScrollerOhos` from a speed the platform
//! injects. [`Coasting`] rebuilds that inertia with the same decay curve and the same parameters,
//! so that a touchpad scroll keeps gliding in the port.
//!
//! `docs/s0/arkweb-inertia-research.md` is the source of every number quoted below: it reads them
//! out of `chromium_ext/ui/gl/events/native_scroller_ohos.cc` in the ArkWeb fork, and
//! `docs/s0/ohos-coasting-feasibility.md` §5.3 picks this file as the landing point. Horizontal
//! coasting is deliberately left out, because ArkWeb only implements the vertical one
//! (`arkweb-inertia-research.md:217`, `:360`).
//!
//! Both halves are pure: they never read a clock, never touch the platform and only use `std`, so
//! their behaviour is unit-tested on the host. The event time always travels with the sample, which
//! is what keeps the estimate and the animation deterministic.
//!
//! # Sign and unit convention
//!
//! [`wheel_delta_from_axis`] mirrors both axis sources exactly once. [`Coasting`] is fed the
//! touchpad axis value as `OH_ArkUI_AxisEvent_GetVerticalAxisValue()` reports it: in the frame of
//! the content, positive when the page advances towards the end of the document (two fingers swiped
//! up). Its output is in the frame of `WheelDelta::y`, mirrored the same single way, so a gesture
//! hands over to the animation without a jump: the frame after the last touch moves the viewport the
//! same way as the last touch did. Speeds are in pixels per second, the unit ArkWeb compares its
//! thresholds against.

// Everything in this file is used by the OpenHarmony embedder and by the tests at the bottom, so a
// host build that does not run the tests sees the whole module as dead code. One expectation covers
// every item, which keeps the host gate at zero warnings.
#![cfg_attr(
    not(any(test, target_env = "ohos")),
    expect(
        dead_code,
        reason = "only the OpenHarmony embedder converts axis events and coasts so far"
    )
)]

use servo::{WheelDelta, WheelMode};

/// The kind of device that produced a scroll axis event.
///
/// The two kinds report their axis values in *different units*, but both are positive in the same
/// direction: the one the content is given, i.e. the direction in which the page advances towards
/// the end of the document. Only the unit, and therefore the scale, differs, see
/// [`wheel_delta_from_axis`].
#[cfg_attr(
    not(any(test, target_env = "ohos")),
    expect(
        dead_code,
        reason = "only the OpenHarmony embedder converts axis events so far"
    )
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AxisEventSource {
    /// A mouse wheel, reporting an angle in degrees.
    MouseWheel,
    /// A touchpad, reporting a distance in pixels.
    Touchpad,
}

/// Angular size of a single mouse wheel notch, in degrees.
///
/// The OpenHarmony axis event reports mouse wheel scrolling as an angular increment in degrees
/// (see `OH_ArkUI_AxisEvent_GetVerticalAxisValue()`), including the scroll step that the user
/// configured. One notch is 15 degrees, following the `WHEEL_DELTA` convention of 120 units per
/// notch with 8 units per degree.
const DEGREES_PER_WHEEL_NOTCH: f32 = 15.0;

/// Height of a single line of scrolled content, in device pixels.
///
/// The desktop port converts a line based wheel delta with the same line height, so that one notch
/// scrolls the same distance in every embedder.
const LINE_HEIGHT: f32 = 76.0;

/// Converts the axis values of a scroll event into a [`WheelDelta`] in device pixels.
///
/// [`AxisEventSource`] decides the unit, and therefore the factor each axis value is scaled by:
///
/// * A mouse wheel reports an angle in degrees that already includes the user's scroll step, so one
///   notch scrolls as much as one line in the desktop port, i.e. [`LINE_HEIGHT`].
/// * A touchpad reports a distance in pixels, which is used as is. Keeping the delta as small as the
///   finger movement is what makes a touchpad scroll smoothly.
///
/// Both sources describe the motion of the *content*, while [`WheelDelta`] describes the motion of
/// the *viewport*: the two are opposite, so the scaled values are mirrored exactly once, for every
/// source alike. Two devices that report the same direction therefore scroll the viewport the same
/// way. This is the only place where the sign is flipped; servo mirrors the delta once more when it
/// turns the wheel event into a scroll.
#[cfg_attr(
    not(any(test, target_env = "ohos")),
    expect(
        dead_code,
        reason = "only the OpenHarmony embedder converts axis events so far"
    )
)]
pub(crate) fn wheel_delta_from_axis(
    horizontal: f32,
    vertical: f32,
    source: AxisEventSource,
) -> WheelDelta {
    let (horizontal, vertical) = match source {
        AxisEventSource::MouseWheel => (
            (horizontal / DEGREES_PER_WHEEL_NOTCH) * LINE_HEIGHT,
            (vertical / DEGREES_PER_WHEEL_NOTCH) * LINE_HEIGHT,
        ),
        AxisEventSource::Touchpad => (horizontal, vertical),
    };
    // The single sign flip that the note above describes, shared by every source.
    WheelDelta {
        x: -horizontal as f64,
        y: -vertical as f64,
        z: 0.0,
        mode: WheelMode::DeltaPixel,
    }
}

/// Friction of a fast two-finger touchpad fling, negative.
///
/// ArkWeb `NativeScrollerOhos`: `kTouchpadFriction (1.2) * kFrictionScale (-4.2) = -5.04`
/// (`docs/s0/arkweb-inertia-research.md:198-201`, applied at `:212`).
pub(crate) const FRICTION_TOUCHPAD: f64 = -5.04;

/// Friction ArkWeb falls back to when a fling starts below [`START_VELOCITY`].
///
/// Same source: `kSlowFriction (1.0) * kFrictionScale (-4.2) = -4.2` (`:202`). ArkWeb picks it with
/// `if (init_velocity_y_ < kStartVelocityThreshold * kVelocityScale) friction_ = kSlowFriction *
/// kFrictionScale;` (`native_scroller_ohos.cc`, quoted at `:218-219`): a slow fling is **not**
/// refused, it just decays with the weaker friction.
pub(crate) const FRICTION_SLOW: f64 = -4.2;

/// Speed below which ArkWeb flings with [`FRICTION_SLOW`] instead of [`FRICTION_TOUCHPAD`], in
/// pixels per second.
///
/// ArkWeb: `kStartVelocityThreshold (1500) * kVelocityScale (1.2) = 1800` (`:203-204`, used at
/// `:218-219`). This is **not** a launch gate: a slower gesture still coasts, it only decays more
/// gently. Whether an animation starts at all is decided against the stop speed ([`END_VELOCITY`]
/// over the virtual pixel ratio), because a fling launched below it could not move anything on its
/// very first frame.
pub(crate) const START_VELOCITY: f64 = 1800.0;

/// Speed at which a coasting animation is considered finished, in pixels per second.
///
/// ArkWeb: `kDefaultThreshold (2.0) * kDefaultMultiplier (60.0) = 120` (`:205-206`), then
/// divided by the virtual pixel ratio (`:221-222`).
pub(crate) const END_VELOCITY: f64 = 120.0;

/// Remaining distance under which a coasting animation is finished, in pixels.
///
/// ArkWeb: `kThresholdForFlingEnd = 1.0` (`:207`, used by `ShouldAbortAnimation` at `:229`).
pub(crate) const END_RESIDUAL: f64 = 1.0;

/// Speed clamp, in pixels per second.
///
/// ArkWeb clamps a fling's velocity to ±9000 on PC devices (`kMaxBoostFlingSpeed`, `:273`,
/// repeated in the migration table at `:392`). Here it also keeps a single absurd sample
/// from throwing the page.
pub(crate) const MAX_VELOCITY: f64 = 9000.0;

/// Length of the gesture tail used to estimate the launch speed, in seconds.
///
/// ArkWeb estimates its launch speed elsewhere (`ui::FlingBooster` looks at the last 50 ms,
/// `kFlingBoostTimeoutDelay`, `docs/s0/arkweb-inertia-research.md:267`); this module uses
/// the 100 ms tail that its design calls for, which also covers the touchpad event rate.
pub(crate) const VELOCITY_WINDOW: f64 = 0.1;

/// Upper bound on the number of stored tail samples.
const MAX_SAMPLES: usize = 64;

/// The device that produced an axis gesture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CoastingTool {
    /// Two-finger swipe on the touchpad. The only source that may start coasting.
    Touchpad,
    /// Mouse wheel. A notch is a discrete, already rate-limited step, so it never coasts,
    /// which is what the ports did before coasting existed.
    MouseWheel,
}

/// The phase of an axis gesture, mirroring `UI_AXIS_EVENT_ACTION_*`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AxisPhase {
    /// `UI_AXIS_EVENT_ACTION_BEGIN = 1`.
    Begin,
    /// `UI_AXIS_EVENT_ACTION_UPDATE = 2`.
    Update,
    /// `UI_AXIS_EVENT_ACTION_END = 3`.
    End,
    /// `UI_AXIS_EVENT_ACTION_CANCEL = 4`. A cancelled gesture only brakes, it never coasts.
    Cancel,
}

/// Every knob of the coasting animation, in one place so a port can tune it on device.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CoastingConfig {
    /// Friction of a fast touchpad fling, negative: [`FRICTION_TOUCHPAD`].
    pub(crate) friction_touchpad: f64,
    /// Friction of a fling that starts slowly: [`FRICTION_SLOW`].
    pub(crate) friction_slow: f64,
    /// Speed below which a fling decays with `friction_slow`: [`START_VELOCITY`].
    pub(crate) start_velocity: f64,
    /// Speed at which the animation stops, before the DPR division: [`END_VELOCITY`].
    pub(crate) end_velocity: f64,
    /// Remaining distance at which the animation stops: [`END_RESIDUAL`].
    pub(crate) end_residual: f64,
    /// Speed clamp: [`MAX_VELOCITY`].
    pub(crate) max_velocity: f64,
    /// Length of the gesture tail used to estimate the launch speed: [`VELOCITY_WINDOW`].
    pub(crate) velocity_window: f64,
    /// Virtual pixel ratio. ArkWeb divides its stop speed by it (`:221-222`).
    pub(crate) device_pixel_ratio: f64,
}

impl Default for CoastingConfig {
    fn default() -> Self {
        CoastingConfig {
            friction_touchpad: FRICTION_TOUCHPAD,
            friction_slow: FRICTION_SLOW,
            start_velocity: START_VELOCITY,
            end_velocity: END_VELOCITY,
            end_residual: END_RESIDUAL,
            max_velocity: MAX_VELOCITY,
            velocity_window: VELOCITY_WINDOW,
            device_pixel_ratio: 1.0,
        }
    }
}

impl CoastingConfig {
    /// [`CoastingConfig::end_velocity`] in the frame the samples are measured in.
    fn stop_velocity(&self) -> f64 {
        if self.device_pixel_ratio > 0.0 {
            self.end_velocity / self.device_pixel_ratio
        } else {
            self.end_velocity
        }
    }
}

/// What one frame of a coasting animation asks the viewport to do.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CoastingFrame {
    /// Scroll delta for this frame, in the frame of `WheelDelta::y`.
    pub(crate) delta: f64,
    /// Whether the animation is over. `delta` may still carry the last few pixels.
    pub(crate) finished: bool,
}

/// One `(time, cumulative displacement)` pair of the gesture tail.
#[derive(Clone, Copy, Debug)]
struct Sample {
    /// Seconds since an arbitrary origin; only differences matter.
    time: f64,
    /// Vertical displacement accumulated since the gesture began, in the frame of the
    /// platform axis value.
    displacement: f64,
}

/// A coasting animation in flight.
#[derive(Clone, Copy, Debug)]
struct Fling {
    /// When the animation started, on the same clock as the samples.
    start: f64,
    /// Time of the previous frame.
    last: f64,
    /// Time at which the animation is due to end.
    end: f64,
    /// Signed launch speed, in the platform frame and in pixels per second.
    velocity: f64,
    /// Friction, negative.
    friction: f64,
}

/// Turns an axis gesture into a coasting animation.
///
/// Feed it every axis event with [`Coasting::feed`], and ask it for one frame's worth of
/// motion with [`Coasting::advance`] on each frame, passing the frame's deadline as the
/// same clock the samples were stamped with.
#[derive(Clone, Debug)]
pub(crate) struct Coasting {
    config: CoastingConfig,
    /// Tail of the gesture being tracked.
    samples: Vec<Sample>,
    /// Displacement accumulated during the current gesture.
    displacement: f64,
    /// Time of the last accepted sample, used to reject out-of-order timestamps.
    last_sample_time: Option<f64>,
    /// Device that produced the current gesture; `None` between gestures.
    tool: Option<CoastingTool>,
    /// The animation in flight, if any.
    fling: Option<Fling>,
}

impl Coasting {
    /// A coasting animation with ArkWeb's parameters.
    pub(crate) fn new() -> Self {
        Self::with_config(CoastingConfig::default())
    }

    /// A coasting animation with the given parameters, for tuning on device.
    pub(crate) fn with_config(config: CoastingConfig) -> Self {
        Coasting {
            config,
            samples: Vec::new(),
            displacement: 0.0,
            last_sample_time: None,
            tool: None,
            fling: None,
        }
    }

    /// Whether an animation is in flight.
    pub(crate) fn is_active(&self) -> bool {
        self.fling.is_some()
    }

    /// Stops any animation in flight and forgets the gesture being tracked.
    ///
    /// This is the idempotent `stop()`: calling it again, including before the first
    /// gesture, changes nothing.
    pub(crate) fn cancel(&mut self) {
        self.fling = None;
        self.reset_gesture();
    }

    /// The speed the running animation was launched with, in pixels per second, in the frame of
    /// the platform axis value; `0.0` when nothing is running.
    ///
    /// Read-only accessor: the unit tests assert on it, and an embedder can read it to tune the
    /// parameters on a device. The animation itself never reads it back.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "read by the unit tests and available to embedders tuning the parameters"
        )
    )]
    pub(crate) fn velocity(&self) -> f64 {
        self.fling.map_or(0.0, |fling| fling.velocity)
    }

    /// Feeds one axis event.
    ///
    /// `time_ns` is the event time from the platform (`OH_ArkUI_UIInputEvent_GetEventTime`,
    /// in nanoseconds), and `delta` is the vertical axis value in the frame of the platform
    /// axis event, the same value that [`wheel_delta_from_axis`] converts. `phase` mirrors
    /// `UI_AXIS_EVENT_ACTION_*` and `tool` mirrors `OH_ArkUI_UIInputEvent_GetToolType`.
    pub(crate) fn feed(&mut self, time_ns: i64, delta: f32, tool: CoastingTool, phase: AxisPhase) {
        let time = time_ns as f64 * 1e-9;
        if !time.is_finite() {
            return;
        }
        match phase {
            // Touching the touchpad again brakes, the way ArkWeb's coasting `END` stops the
            // animation when the user re-engages the touchpad. A new gesture must also never
            // inherit the speed of the previous one.
            AxisPhase::Begin => {
                self.fling = None;
                self.reset_gesture();
                self.tool = Some(tool);
                self.record(time, delta);
            },
            AxisPhase::Update => {
                if self.fling.is_some() {
                    // An update arriving during an animation means the platform is driving
                    // again: give the animation up rather than adding motion to motion.
                    self.fling = None;
                    self.reset_gesture();
                }
                if self.tool.is_none() {
                    self.tool = Some(tool);
                }
                self.record(time, delta);
            },
            // Only a gesture that *ended* can launch an animation. The platform reports a
            // `Cancel` when the gesture was taken away from us instead of being finished, and a
            // cancelled gesture is not a flick: it brakes whatever is in flight and stops there.
            AxisPhase::End => {
                self.fling = None;
                if self.tool.is_none() {
                    self.tool = Some(tool);
                }
                self.record(time, delta);
                self.launch(time);
                self.reset_gesture();
            },
            AxisPhase::Cancel => {
                self.fling = None;
                self.reset_gesture();
            },
        }
    }

    /// Advances the animation to `now_ns` and returns what this frame should scroll.
    ///
    /// The position comes from the closed form of the decay, so it does not depend on how
    /// often this is called: a late frame carries the distance of the frames it replaces.
    /// Frames that arrive out of order or twice produce no motion, and a finished (or
    /// never started) animation asks for nothing.
    pub(crate) fn advance(&mut self, now_ns: i64) -> CoastingFrame {
        let now = now_ns as f64 * 1e-9;
        let Some(fling) = self.fling else {
            return CoastingFrame {
                delta: 0.0,
                finished: true,
            };
        };
        // A non-negative or non-finite friction would not decay: refuse it instead of
        // producing a non-finite delta.
        if !fling.friction.is_finite() || fling.friction >= 0.0 {
            self.fling = None;
            return CoastingFrame {
                delta: 0.0,
                finished: true,
            };
        }
        if !now.is_finite() || now <= fling.last {
            // A duplicate or out-of-order frame: the clock did not move.
            return CoastingFrame {
                delta: 0.0,
                finished: false,
            };
        }

        let mut fling = fling;
        let from = (fling.last - fling.start).clamp(0.0, fling.end);
        let to = (now - fling.start).clamp(0.0, fling.end);
        // ArkWeb: `GetPosition(t) = init_y + sign * (|v0| / friction) * expm1(friction * t)`,
        // with `init_y = 0` (`docs/s0/arkweb-inertia-research.md:227`).
        let position =
            |time: f64| fling.velocity / fling.friction * (fling.friction * time).exp_m1();
        let platform_delta = position(to) - position(from);
        let velocity = fling.velocity * (fling.friction * to).exp();
        let remaining = (velocity / fling.friction).abs();
        // ArkWeb `ShouldAbortAnimation()` (`:229-230`), plus the end of the planned decay.
        let finished = to >= fling.end
            || velocity.abs() <= self.config.stop_velocity()
            || remaining < self.config.end_residual;

        fling.last = now;
        self.fling = (!finished).then_some(fling);

        CoastingFrame {
            // The single mirror that `wheel_delta_from_axis()` applies to both axis sources,
            // so the animation carries on in the direction of the gesture.
            delta: -platform_delta,
            finished,
        }
    }

    /// Clears the gesture tail, keeping the animation (if any) alone.
    fn reset_gesture(&mut self) {
        self.samples.clear();
        self.displacement = 0.0;
        self.last_sample_time = None;
        self.tool = None;
    }

    /// Appends a sample, dropping anything that would make the time axis non-monotonic.
    fn record(&mut self, time: f64, delta: f32) {
        let delta = delta as f64;
        if !delta.is_finite() {
            return;
        }
        if let Some(last) = self.last_sample_time {
            // Out-of-order timestamps, duplicates and `dt == 0` are dropped rather than
            // divided by, so a bad sample can neither panic nor distort the estimate.
            if time <= last {
                return;
            }
        }
        self.displacement += delta;
        self.last_sample_time = Some(time);
        self.samples.push(Sample {
            time,
            displacement: self.displacement,
        });
        self.prune(time);
    }

    /// Keeps only the tail of the gesture that the launch speed is estimated from.
    fn prune(&mut self, now: f64) {
        let keep_from = now - self.config.velocity_window;
        // Keep at least one sample, so that a window shorter than the spacing between two
        // events still leaves an origin behind.
        while self.samples.len() > 1 {
            let too_old = self
                .samples
                .first()
                .is_some_and(|sample| sample.time < keep_from);
            if !too_old && self.samples.len() <= MAX_SAMPLES {
                break;
            }
            self.samples.remove(0);
        }
    }

    /// Average speed over the gesture tail, in pixels per second, or `None` when the
    /// gesture is too short to hold a speed.
    fn estimate_velocity(&self) -> Option<f64> {
        let first = self.samples.first()?;
        let last = self.samples.last()?;
        let elapsed = last.time - first.time;
        if elapsed <= 0.0 {
            // A single sample, or several stamped at the same instant: no speed.
            return None;
        }
        let velocity = (last.displacement - first.displacement) / elapsed;
        if !velocity.is_finite() {
            return None;
        }
        Some(velocity.clamp(-self.config.max_velocity, self.config.max_velocity))
    }

    /// Launches the animation of the gesture that just ended, if the gesture is a flick at all.
    fn launch(&mut self, now: f64) {
        // A wheel notch is not a gesture, so it never coasts.
        if self.tool != Some(CoastingTool::Touchpad) {
            return;
        }
        let Some(velocity) = self.estimate_velocity() else {
            return;
        };
        // ArkWeb does not refuse a slow fling: it only gives it the weaker friction
        // (`docs/s0/arkweb-inertia-research.md:218-219`). What is refused here is a speed that the
        // animation would give up on immediately — below the stop speed its first frame is already
        // its last one, so there is nothing to start.
        if velocity.abs() <= self.config.stop_velocity() {
            return;
        }
        let friction = self.friction_for(velocity);
        self.fling = Some(Fling {
            start: now,
            last: now,
            end: end_time(velocity, self.config.stop_velocity(), friction),
            velocity,
            friction,
        });
    }

    /// The friction ArkWeb would use for this launch speed: the slow one below [`START_VELOCITY`],
    /// the touchpad one above it (`:218-219`).
    fn friction_for(&self, velocity: f64) -> f64 {
        if velocity.abs() < self.config.start_velocity {
            self.config.friction_slow
        } else {
            self.config.friction_touchpad
        }
    }
}

/// Time at which an exponential decay reaches the stop speed.
///
/// ArkWeb: `diff_time = log(velocity_threshold / init_velocity) / friction`
/// (`docs/s0/arkweb-inertia-research.md:224`). For the speeds that reach this far the log is
/// of a value below one and the friction is negative, so the result is positive.
fn end_time(velocity: f64, stop: f64, friction: f64) -> f64 {
    if !friction.is_finite() || friction >= 0.0 || stop <= 0.0 {
        return 0.0;
    }
    let time = (stop / velocity.abs()).ln() / friction;
    if time.is_finite() { time.max(0.0) } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use servo::WheelMode;

    use super::MAX_VELOCITY;
    use super::{AxisEventSource, DEGREES_PER_WHEEL_NOTCH, LINE_HEIGHT, wheel_delta_from_axis};
    use super::{
        AxisPhase, Coasting, CoastingTool, END_VELOCITY, FRICTION_SLOW, FRICTION_TOUCHPAD,
    };

    const WHEEL: AxisEventSource = AxisEventSource::MouseWheel;
    const TOUCHPAD: AxisEventSource = AxisEventSource::Touchpad;

    #[test]
    fn a_wheel_notch_scrolls_one_line() {
        // A mouse wheel value is an angle in degrees. Scroll forward one notch and one line is
        // scrolled, which is the same distance that the desktop port scrolls for one notch.
        let delta = wheel_delta_from_axis(0.0, -DEGREES_PER_WHEEL_NOTCH, WHEEL);
        assert_eq!(delta.mode, WheelMode::DeltaPixel);
        assert_eq!(delta.x, 0.0);
        assert_eq!(delta.y, LINE_HEIGHT as f64);
        assert_eq!(delta.z, 0.0);

        // Scrolling backwards is positive and moves the content up, i.e. the viewport down.
        assert_eq!(
            wheel_delta_from_axis(0.0, DEGREES_PER_WHEEL_NOTCH, WHEEL).y,
            -(LINE_HEIGHT as f64)
        );

        // The horizontal axis is mirrored the same way.
        assert_eq!(
            wheel_delta_from_axis(DEGREES_PER_WHEEL_NOTCH, 0.0, WHEEL).x,
            -(LINE_HEIGHT as f64)
        );

        // The user scroll step multiplies the reported angle, so the delta scales with it.
        assert_eq!(
            wheel_delta_from_axis(0.0, 2.0 * DEGREES_PER_WHEEL_NOTCH, WHEEL).y,
            -2.0 * LINE_HEIGHT as f64
        );
    }

    #[test]
    fn a_touchpad_value_is_a_distance_in_pixels() {
        // A touchpad value is the distance that the fingers travelled, in pixels. It is not
        // converted with the wheel angle, so the delta is not magnified by about five times.
        let delta = wheel_delta_from_axis(30.0, -57.0, TOUCHPAD);
        assert_eq!(delta.mode, WheelMode::DeltaPixel);
        assert_eq!(delta.x, -30.0);
        assert_eq!(delta.y, 57.0);

        // Small movements stay small, which is what makes a touchpad scroll smoothly.
        assert_eq!(wheel_delta_from_axis(0.0, 2.0, TOUCHPAD).y, -2.0);
        assert_eq!(wheel_delta_from_axis(-3.0, 0.0, TOUCHPAD).x, 3.0);
    }

    #[test]
    fn both_sources_are_mirrored_once() {
        // Both devices speak in the frame of the content: swiping up is positive for the platform,
        // and so is the wheel rotation that the header calls backward scrolling. The content follows
        // the input, so a positive value scrolls the viewport down.
        assert!(wheel_delta_from_axis(0.0, 40.0, TOUCHPAD).y < 0.0);
        assert!(wheel_delta_from_axis(0.0, -40.0, TOUCHPAD).y > 0.0);
        // Swiping right is positive for the platform and scrolls the viewport to the left.
        assert!(wheel_delta_from_axis(40.0, 0.0, TOUCHPAD).x < 0.0);
        assert!(wheel_delta_from_axis(-40.0, 0.0, TOUCHPAD).x > 0.0);

        // The wheel is mirrored exactly like the touchpad, on both axes.
        assert!(wheel_delta_from_axis(0.0, DEGREES_PER_WHEEL_NOTCH, WHEEL).y < 0.0);
        assert!(wheel_delta_from_axis(0.0, -DEGREES_PER_WHEEL_NOTCH, WHEEL).y > 0.0);
        assert!(wheel_delta_from_axis(DEGREES_PER_WHEEL_NOTCH, 0.0, WHEEL).x < 0.0);
        assert!(wheel_delta_from_axis(-DEGREES_PER_WHEEL_NOTCH, 0.0, WHEEL).x > 0.0);
    }

    #[test]
    fn a_backward_wheel_notch_and_an_upward_touchpad_swipe_agree() {
        // Regression test for the reported defect: the mouse wheel scrolled the opposite way round
        // from every other application, because only the touchpad was mirrored. A wheel turned
        // backwards (positive degrees) and two fingers swiped up (positive pixels) are the same
        // request from two devices, so they must move the viewport the same way.
        let from_wheel = wheel_delta_from_axis(0.0, DEGREES_PER_WHEEL_NOTCH, WHEEL);
        let from_touchpad = wheel_delta_from_axis(0.0, 40.0, TOUCHPAD);
        assert!(from_wheel.y < 0.0 && from_touchpad.y < 0.0);
        assert_eq!(from_wheel.mode, from_touchpad.mode);

        // And so must the opposite request: a wheel turned forwards and two fingers swept down.
        let from_wheel = wheel_delta_from_axis(0.0, -DEGREES_PER_WHEEL_NOTCH, WHEEL);
        let from_touchpad = wheel_delta_from_axis(0.0, -40.0, TOUCHPAD);
        assert!(from_wheel.y > 0.0 && from_touchpad.y > 0.0);
        assert_eq!(from_wheel.mode, from_touchpad.mode);
    }

    #[test]
    fn zero_and_tiny_values_do_not_jump() {
        // A zero axis value produces no movement, whatever the unit of the source.
        for source in [WHEEL, TOUCHPAD] {
            let delta = wheel_delta_from_axis(0.0, 0.0, source);
            assert_eq!(delta.x, 0.0);
            assert_eq!(delta.y, 0.0);
            assert_eq!(delta.z, 0.0);
            assert_eq!(delta.mode, WheelMode::DeltaPixel);
        }

        // A fraction of a notch is a fraction of a line, and a pixel stays a pixel.
        assert!(wheel_delta_from_axis(0.0, -0.2, WHEEL).y > 0.0);
        assert!(wheel_delta_from_axis(0.0, -0.2, WHEEL).y < LINE_HEIGHT as f64);
        assert_eq!(wheel_delta_from_axis(0.0, 0.5, TOUCHPAD).y, -0.5);
    }

    /// Milliseconds in nanoseconds, for readable sample times.
    const MS: i64 = 1_000_000;
    /// One frame at 60 Hz, in nanoseconds.
    const FRAME: i64 = 16_666_667;

    const COAST_TOUCHPAD: CoastingTool = CoastingTool::Touchpad;
    const COAST_WHEEL: CoastingTool = CoastingTool::MouseWheel;

    /// Plays a gesture: `samples` are `(time_ns, delta)` pairs from `Begin` to the last
    /// `Update`, and the gesture ends at `end_ns` with no further movement.
    fn play(coasting: &mut Coasting, tool: CoastingTool, samples: &[(i64, f32)], end_ns: i64) {
        for (index, (time_ns, delta)) in samples.iter().enumerate() {
            let phase = if index == 0 {
                AxisPhase::Begin
            } else {
                AxisPhase::Update
            };
            coasting.feed(*time_ns, *delta, tool, phase);
        }
        coasting.feed(end_ns, 0.0, tool, AxisPhase::End);
    }

    /// A 20 px per 10 ms swipe: 2000 px/s, above the 1800 px/s launch threshold.
    fn fast_swipe() -> Vec<(i64, f32)> {
        vec![(0, 0.0), (10 * MS, 20.0), (20 * MS, 20.0), (30 * MS, 20.0)]
    }

    #[test]
    fn the_decay_is_monotonic_and_stops_at_the_stop_speed() {
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &fast_swipe(), 30 * MS);
        assert!(coasting.is_active());

        let start = 30 * MS;
        let mut previous = f64::INFINITY;
        let mut travelled = 0.0;
        let mut finished_after = None;
        for frame in 1..=200 {
            let step = coasting.advance(start + frame * FRAME);
            let distance = step.delta.abs();
            assert!(distance.is_finite());
            assert!(distance <= previous, "the decay must not speed up again");
            travelled += distance;
            previous = distance;
            if step.finished {
                finished_after = Some(frame);
                break;
            }
        }

        let finished_after = finished_after.expect("the fling must end on its own");
        assert!(!coasting.is_active());
        // The decay covers `(launch - stop) / |friction|` pixels, in
        // `log(stop / launch) / friction` seconds.
        let expected = (2000.0 - END_VELOCITY) / -FRICTION_TOUCHPAD;
        assert!(
            (travelled - expected).abs() < 0.5,
            "{travelled} != {expected}"
        );
        let seconds = finished_after as f64 * FRAME as f64 * 1e-9;
        let expected_time = (END_VELOCITY / 2000.0f64).ln() / FRICTION_TOUCHPAD;
        assert!(
            (seconds - expected_time).abs() < 0.05,
            "{seconds} != {expected_time}"
        );

        // Once it is over, nothing more moves.
        let after = coasting.advance(start + 10_000 * FRAME);
        assert_eq!(after.delta, 0.0);
        assert!(after.finished);
    }

    #[test]
    fn a_slow_gesture_coasts_with_the_slow_friction() {
        // 5 px per 10 ms is 500 px/s: above the stop speed, but below ArkWeb's 1800 px/s slow
        // threshold, so the fling decays with `FRICTION_SLOW` (`:218-219`) instead of being
        // refused, and travels much less than a fast one.
        let samples = [(0, 0.0), (10 * MS, 5.0), (20 * MS, 5.0), (30 * MS, 5.0)];
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &samples, 30 * MS);
        assert!(coasting.is_active());

        // One 60 Hz frame later the position follows the same closed form as the fast gesture,
        // with the slow friction.
        let frame = coasting.advance(30 * MS + FRAME);
        let t = FRAME as f64 * 1e-9;
        let expected = (500.0 / FRICTION_SLOW) * (FRICTION_SLOW * t).exp_m1();
        assert!(!frame.finished);
        assert!(
            (frame.delta + expected).abs() < 1e-6,
            "{} != {}",
            frame.delta,
            -expected
        );

        let mut travelled = frame.delta.abs();
        for frame in 2..=200 {
            let step = coasting.advance(30 * MS + frame * FRAME);
            travelled += step.delta.abs();
            if step.finished {
                break;
            }
        }
        // The whole glide covers `(launch - stop) / |friction|` pixels, and is shorter than the
        // fast one by construction.
        let expected_total = (500.0 - END_VELOCITY) / -FRICTION_SLOW;
        assert!(
            (travelled - expected_total).abs() < 0.5,
            "{travelled} != {expected_total}"
        );
        assert!(travelled < (2000.0 - END_VELOCITY) / -FRICTION_TOUCHPAD);
    }

    #[test]
    fn a_gesture_below_the_stop_speed_does_not_coast() {
        // 1 px per 10 ms is 100 px/s, below the 120 px/s stop speed: the first frame of such an
        // animation would already be its last one, so it is not started at all.
        let samples = [(0, 0.0), (10 * MS, 1.0), (20 * MS, 1.0), (30 * MS, 1.0)];
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &samples, 30 * MS);
        assert!(!coasting.is_active());
        assert_eq!(coasting.velocity(), 0.0);

        let step = coasting.advance(30 * MS + FRAME);
        assert_eq!(step.delta, 0.0);
        assert!(step.finished);
    }

    #[test]
    fn a_fast_gesture_coasts_with_arkwebs_decay() {
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &fast_swipe(), 30 * MS);
        assert!(coasting.is_active());

        // The launch speed is the average over the 100 ms tail: 60 px in 30 ms, so
        // 2000 px/s. One 60 Hz frame later the position is
        // `(v0 / friction) * expm1(friction * t)`, with ArkWeb's touchpad friction.
        let frame = coasting.advance(30 * MS + FRAME);
        let t = FRAME as f64 * 1e-9;
        let expected = (2000.0 / FRICTION_TOUCHPAD) * (FRICTION_TOUCHPAD * t).exp_m1();
        assert!(!frame.finished);
        assert!(
            (frame.delta + expected).abs() < 1e-6,
            "{} != {}",
            frame.delta,
            -expected
        );
        // Fingers up is a positive platform value, and it scrolls the viewport the same way
        // the axis event that started the gesture did.
        assert!(frame.delta < 0.0);
    }

    #[test]
    fn a_new_gesture_brakes_a_fling_in_flight() {
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &fast_swipe(), 30 * MS);
        coasting.advance(30 * MS + FRAME);
        assert!(coasting.is_active());

        // Touching the touchpad again stops the animation, the way ArkWeb's coasting `END`
        // stops it when the user re-engages the touchpad.
        coasting.feed(40 * MS, 0.0, COAST_TOUCHPAD, AxisPhase::Begin);
        assert!(!coasting.is_active());
        let step = coasting.advance(40 * MS + FRAME);
        assert_eq!(step.delta, 0.0);
        assert!(step.finished);

        // `cancel` is idempotent and leaves the same resting state.
        coasting.cancel();
        coasting.cancel();
        assert!(!coasting.is_active());
        assert_eq!(coasting.advance(40 * MS + 2 * FRAME).delta, 0.0);
    }

    #[test]
    fn a_wheel_never_coasts() {
        // The same fast gesture, reported by a mouse wheel: a notch is a discrete step, and
        // the port keeps scrolling them one notch at a time.
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_WHEEL, &fast_swipe(), 30 * MS);
        assert!(!coasting.is_active());
        assert!(coasting.advance(30 * MS + FRAME).finished);
    }

    #[test]
    fn degenerate_samples_are_safe() {
        // A single sample cannot hold a speed.
        let mut single = Coasting::new();
        play(&mut single, COAST_TOUCHPAD, &[(7 * MS, 40.0)], 7 * MS);
        assert!(!single.is_active());

        // Samples stamped at the same instant are dropped: no division by zero.
        let mut repeated = Coasting::new();
        play(
            &mut repeated,
            COAST_TOUCHPAD,
            &[(0, 0.0), (0, 40.0), (0, 40.0)],
            0,
        );
        assert!(!repeated.is_active());

        // An out-of-order timestamp is dropped instead of shortening the window.
        let mut out_of_order = Coasting::new();
        out_of_order.feed(100 * MS, 0.0, COAST_TOUCHPAD, AxisPhase::Begin);
        out_of_order.feed(50 * MS, 999.0, COAST_TOUCHPAD, AxisPhase::Update);
        out_of_order.feed(110 * MS, 20.0, COAST_TOUCHPAD, AxisPhase::Update);
        out_of_order.feed(110 * MS, 0.0, COAST_TOUCHPAD, AxisPhase::End);
        assert!(out_of_order.is_active());

        // An absurd single-frame delta is clamped instead of throwing the page.
        let mut huge = Coasting::new();
        play(
            &mut huge,
            COAST_TOUCHPAD,
            &[(0, 0.0), (10 * MS, 1.0e9)],
            10 * MS,
        );
        assert!(huge.is_active());
        let step = huge.advance(10 * MS + FRAME);
        assert!(step.delta.is_finite());
        assert!(step.delta.abs() <= MAX_VELOCITY * FRAME as f64 * 1e-9 + 1.0);

        // A frame that arrives out of order produces no motion and does not end the fling.
        let on_time = huge.advance(10 * MS + 2 * FRAME);
        assert!(on_time.delta.is_finite());
        let backwards = huge.advance(10 * MS + FRAME);
        assert_eq!(backwards.delta, 0.0);
        assert!(!backwards.finished);
    }

    #[test]
    fn the_coasting_direction_matches_the_axis_convention() {
        // Regression guard for the direction the user already validated: `wheel_delta_from_axis()`
        // mirrors both axis sources once (`both_sources_are_mirrored_once` above pins the same
        // convention for the axis events themselves), so the first coasting frame of an upward
        // swipe must have the same sign as that mirror, and a downward swipe the opposite one.
        let up = [(0, 0.0), (10 * MS, 20.0), (20 * MS, 20.0), (30 * MS, 20.0)];
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &up, 30 * MS);
        assert!(coasting.advance(30 * MS + FRAME).delta < 0.0);

        let down = [
            (0, 0.0),
            (10 * MS, -20.0),
            (20 * MS, -20.0),
            (30 * MS, -20.0),
        ];
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &down, 30 * MS);
        assert!(coasting.advance(30 * MS + FRAME).delta > 0.0);
    }

    #[test]
    fn a_cancelled_gesture_does_not_coast() {
        // A fast gesture that the platform ends with a `Cancel` was taken away from us instead
        // of being flicked, so nothing is thrown.
        let mut coasting = Coasting::new();
        for (index, (time_ns, delta)) in fast_swipe().iter().enumerate() {
            let phase = if index == 0 {
                AxisPhase::Begin
            } else {
                AxisPhase::Update
            };
            coasting.feed(*time_ns, *delta, COAST_TOUCHPAD, phase);
        }
        coasting.feed(30 * MS, 0.0, COAST_TOUCHPAD, AxisPhase::Cancel);
        assert!(!coasting.is_active());
        assert_eq!(coasting.velocity(), 0.0);
        assert!(coasting.advance(30 * MS + FRAME).finished);
    }

    #[test]
    fn a_cancelled_gesture_brakes_a_fling_in_flight() {
        let mut coasting = Coasting::new();
        play(&mut coasting, COAST_TOUCHPAD, &fast_swipe(), 30 * MS);
        coasting.advance(30 * MS + FRAME);
        assert!(coasting.is_active());
        // The whole tail was inside the window, so the launch speed came from all of it.
        assert!(coasting.velocity() > 0.0);

        coasting.feed(40 * MS, 0.0, COAST_TOUCHPAD, AxisPhase::Cancel);
        assert!(!coasting.is_active());
        assert_eq!(coasting.velocity(), 0.0);
        assert!(coasting.advance(40 * MS + FRAME).finished);
    }
}
