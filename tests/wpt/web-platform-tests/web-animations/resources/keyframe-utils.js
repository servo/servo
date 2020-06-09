'use strict';

// =======================================
//
// Utility functions for testing keyframes
//
// =======================================


// ------------------------------
//  Helper functions
// ------------------------------

/**
 * Test equality between two lists of computed keyframes
 * @param {Array.<ComputedKeyframe>} a - actual computed keyframes
 * @param {Array.<ComputedKeyframe>} b - expected computed keyframes
 */
function assert_frame_lists_equal(a, b) {
  assert_equals(a.length, b.length, 'number of frames');
  for (let i = 0; i < Math.min(a.length, b.length); i++) {
    assert_frames_equal(a[i], b[i], `ComputedKeyframe #${i}`);
  }
}

/** Helper for assert_frame_lists_equal */
function assert_frames_equal(a, b, name) {
  assert_equals(Object.keys(a).sort().toString(),
                Object.keys(b).sort().toString(),
                `properties on ${name} should match`);
  // Iterates sorted keys to ensure stable failures.
  for (const p of Object.keys(a).sort()) {
    assert_equals(a[p], b[p], `value for '${p}' on ${name}`);
  }
}
