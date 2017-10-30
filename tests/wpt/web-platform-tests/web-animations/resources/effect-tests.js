// Common utility methods for testing animation effects

// Tests the |property| member of |animation's| target effect's computed timing
// at the various points indicated by |values|.
//
// |values| has the format:
//
//   {
//     before, // value to test during before phase
//     activeBoundary, // value to test at the very beginning of the active
//                     // phase when playing forwards, or the very end of
//                     // the active phase when playing backwards.
//                     // This should be undefined if the active duration of
//                     // the effect is zero.
//     after,  // value to test during the after phase or undefined if the
//             // active duration is infinite
//   }
//
function assert_computed_timing_for_each_phase(animation, property, values) {
  // Some computed timing properties (e.g. 'progress') require floating-point
  // comparison, whilst exact equality suffices for others.
  const assert_property_equals =
      (property === 'progress') ? assert_times_equal : assert_equals;

  const effect = animation.effect;
  const timing = effect.getComputedTiming();

  // The following calculations are based on the definitions here:
  // https://w3c.github.io/web-animations/#animation-effect-phases-and-states
  const beforeActive = Math.max(Math.min(timing.delay, timing.endTime), 0);
  const activeAfter =
    Math.max(Math.min(timing.delay + timing.activeDuration, timing.endTime), 0);
  const direction = animation.playbackRate < 0 ? 'backwards' : 'forwards';

  // Before phase
  if (direction === 'forwards') {
    animation.currentTime = beforeActive - 1;
  } else {
    animation.currentTime = beforeActive;
  }
  assert_property_equals(effect.getComputedTiming()[property], values.before,
                         `Value of ${property} in the before phase`);

  // Active phase
  if (effect.getComputedTiming().activeDuration > 0) {
    if (direction === 'forwards') {
      animation.currentTime = beforeActive;
    } else {
      animation.currentTime = activeAfter;
    }
    assert_property_equals(effect.getComputedTiming()[property], values.activeBoundary,
                           `Value of ${property} at the boundary of the active phase`);
  } else {
    assert_equals(values.activeBoundary, undefined,
                  'Test specifies a value to check during the active phase but'
                  + ' the animation has a zero duration');
  }

  // After phase
  if (effect.getComputedTiming().activeDuration !== Infinity) {
    if (direction === 'forwards') {
      animation.currentTime = activeAfter;
    } else {
      animation.currentTime = activeAfter + 1;
    }
    assert_property_equals(effect.getComputedTiming()[property], values.after,
                           `Value of ${property} in the after phase`);
  } else {
    assert_equals(values.after, undefined,
                  'Test specifies a value to check during the after phase but'
                  + ' the animation has an infinite duration');
  }
}
