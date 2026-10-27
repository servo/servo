/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Translation of platform scroll input into servo [`WheelDelta`]s.
//!
//! Embedders receive wheel and touchpad scrolling in the unit of their platform, while
//! [`servo::InputEvent::Wheel`] takes a [`WheelDelta`] in device pixels whose `y` is positive when
//! the viewport is scrolled up. Every source reports its values in the frame of the content, which is
//! the opposite of the frame of the viewport, so the arithmetic that bridges the two is the same for
//! all of them and is platform independent: it lives here, outside of the platform specific embedder
//! modules, and is unit tested on the host. Telling the two kinds of input apart is left to the
//! platform module, because the tool types are part of the platform API.

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

#[cfg(test)]
mod tests {
    use servo::WheelMode;

    use super::{AxisEventSource, DEGREES_PER_WHEEL_NOTCH, LINE_HEIGHT, wheel_delta_from_axis};

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
}
