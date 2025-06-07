// Utility assert functions.
// Relies on resources/testharness.js to be included before this file.
// Relies on webxr_test_constants.js to be included before this file.
// Relies on webxr_math_utils.js to be included before this file.


// |p1|, |p2| - objects with x, y, z, w components that are floating point numbers.
// Returns the name of mismatching component between p1 and p2.
const get_mismatched_component = function(p1, p2, epsilon = FLOAT_EPSILON) {
  for (const v of ['x', 'y', 'z', 'w']) {
    if (Math.abs(p1[v] - p2[v]) > epsilon) {
      return v;
    }
  }

  return null;
}

// Internal helper to find the mismatched component for orientations.
// Considers that q and -q represent the same orientation.
// Returns the component name ('x', 'y', 'z', 'w') of the first mismatch
// Returns null if q1 is approximately equal to q2 or -q2.
const get_mismatched_orientation_component = function(q1, q2, epsilon) {
  const direct_mismatch = get_mismatched_component(q1, q2, epsilon);
  // q1 == q2, for our purposes there is no mismatched component.
  if (direct_mismatch === null) {
    return null;
  }
  // q1 != q2, but check q1 vs -q2
  const q2_flipped = flip_quaternion(q2);
  if (get_mismatched_component(q1, q2_flipped, epsilon) === null) {
    return null;
  }
  // q1 is not approx equal to q2 or -q2.
  // both q2 and q2_flipped have non-null mismatchecs, but for ease of debugging
  // return the mismatch from the direct comparison.
  return direct_mismatch;
};

// Internal helper to find the index of the first mismatched matrix element.
// Returns the index (0-15) or -1 if matrices are approximately equal.
const get_mismatched_matrix_element_index = function(m1, m2, epsilon, prefix="") {
  assert_equals(m1.length, 16, prefix + "m1 must have length of 16");
  assert_equals(m2.length, 16, prefix + "m2 must have length of 16");

  for (let i = 0; i < 16; ++i) {
    if (Math.abs(m1[i] - m2[i]) > epsilon) {
      return i;
    }
  }
  return -1;
}

// |p1|, |p2| - objects with x, y, z, w components that are floating point numbers.
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_point_approx_equals = function(p1, p2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (p1 == null && p2 == null) {
    return;
  }

  assert_not_equals(p1, null, prefix + "p1 must be non-null");
  assert_not_equals(p2, null, prefix + "p2 must be non-null");

  const mismatched_component = get_mismatched_component(p1, p2, epsilon);

  if (mismatched_component !== null) {
    let error_message = prefix + ' Point comparison failed.\n';
    error_message += ` p1: {x: ${p1.x}, y: ${p1.y}, z: ${p1.z}, w: ${p1.w}}\n`;
    error_message += ` p2: {x: ${p2.x}, y: ${p2.y}, z: ${p2.z}, w: ${p2.w}}\n`;
    error_message += ` Difference in component ${mismatched_component} exceeded the given epsilon.\n`;
    assert_approx_equals(p2[mismatched_component], p1[mismatched_component], epsilon, error_message);
  }
};

// |p1|, |p2| - objects with x, y, z, w components that are floating point numbers.
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_point_significantly_not_equals = function(p1, p2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (p1 == null || p2 == null) {
    assert_not_equals(p2, p1, prefix + "p1 and p2 are both null");
    return;
  }

  const mismatched_component = get_mismatched_component(p1, p2, epsilon);
  if (mismatched_component === null) {
    let error_message = prefix + ' Point comparison failed (expected significant difference).\n';
    error_message += ` p1: {x: ${p1.x}, y: ${p1.y}, z: ${p1.z}, w: ${p1.w}}\n`;
    error_message += ` p2: {x: ${p2.x}, y: ${p2.y}, z: ${p2.z}, w: ${p2.w}}\n`;
    error_message += ` Difference in components did not exceeded the given epsilon.\n`;
    assert_unreached(error_message);
  }
};

// |q1|, |q2| - objects with x, y, z, w components that are floating point numbers.
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_orientation_approx_equals = function(q1, q2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (q1 == null && q2 == null) {
    return;
  }

  assert_not_equals(q1, null, prefix + "q1 must be non-null");
  assert_not_equals(q2, null, prefix + "q2 must be non-null");

  const mismatched_component = get_mismatched_orientation_component(q1, q2, epsilon);
  // q1 doesn't match neither q2 nor -q2, so it definitely does not represent the same orientations,
  // log an assert failure.
  if (mismatched_component !== null) {
    let error_message = prefix + ' Orientation comparison failed.\n';
    error_message += ` q1: {x: ${q1.x}, y: ${q1.y}, z: ${q1.z}, w: ${q1.w}}\n`;
    error_message += ` q2: {x: ${q2.x}, y: ${q2.y}, z: ${q2.z}, w: ${q2.w}}\n`;
    error_message += ` Neither q2 nor -q2 are approximately equal to q1.\n`;
    error_message += ` For q1 vs q2, difference in component ${mismatched_component} exceeded the given epsilon.\n`;
    assert_approx_equals(q2[mismatched_component], q1[mismatched_component], epsilon, error_message);
  }
};

// |q1|, |q2| - objects with x, y, z, w components that are floating point numbers.
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_orientation_significantly_not_equals = function(q1, q2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (q1 == null || q2 == null) {
    assert_not_equals(q2, q1, prefix + "q1 and q2 are both null");
    return;
  }

  const mismatched_component = get_mismatched_orientation_component(q1, q2, epsilon);
  // IF there is no mismatch q1 matches either q2 or -q2 (which are equivalent).
  if (mismatched_component === null) {
    let error_message = prefix + ' Orientation comparison failed (expected significant difference).\n';
    error_message += ` q1: {x: ${q1.x}, y: ${q1.y}, z: ${q1.z}, w: ${q1.w}}\n`;
    error_message += ` q2: {x: ${q2.x}, y: ${q2.y}, z: ${q2.z}, w: ${q2.w}}\n`;
    error_message += ` q1 is approximately equal to q2 or -q2, but a significant difference was expected.\n`;
    assert_unreached(error_message);
  }
};

// |t1|, |t2| - objects containing position and orientation.
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_transform_approx_equals = function(t1, t2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (t1 == null && t2 == null) {
    return;
  }

  assert_not_equals(t1, null, prefix + "t1 must be non-null");
  assert_not_equals(t2, null, prefix + "t2 must be non-null");

  assert_point_approx_equals(t1.position, t2.position, epsilon, prefix + "positions must be equal");
  assert_orientation_approx_equals(t1.orientation, t2.orientation, epsilon, prefix + "orientations must be equal");
};

// |t1|, |t2| - objects containing position and orientation.
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_transform_significantly_not_equals = function(t1, t2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (t1 == null || t2 == null) {
    assert_not_equals(t1, t2, prefix + "t1 and t2 cannot both be null.");
    return;
  }

  // It is okay for one of position or orientation to be equal; but not for both
  // to be equal in order for the transform to not be equal.
  let mismatched_position = get_mismatched_component(t1.position, t2.position, epsilon);
  let mismatched_orientation = get_mismatched_orientation_component(t1.orientation, t2.orientation, epsilon);
  if (mismatched_position === null && mismatched_orientation === null) {
      assert_point_significantly_not_equals(t1.position, t2.position, epsilon, prefix + "positions must not be equal");
      assert_orientation_significantly_not_equals(t1.orientation, t2.orientation, epsilon, prefix + "orientations must not be equal");
  }
}

// |m1|, |m2| - arrays of floating point numbers
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_matrix_approx_equals = function(m1, m2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (m1 == null && m2 == null) {
    return;
  }

  assert_not_equals(m1, null, prefix + "m1 must be non-null");
  assert_not_equals(m2, null, prefix + "m2 must be non-null");

  assert_equals(m1.length, 16, prefix + "m1 must have length of 16");
  assert_equals(m2.length, 16, prefix + "m2 must have length of 16");

  const mismatched_element = get_mismatched_matrix_element_index(m1, m2, epsilon, prefix);
  if (mismatched_element !== -1) {
    let error_message = prefix + 'Matrix comparison failed.\n';
    error_message += ' Difference in element ' + mismatched_element +
        ' exceeded the given epsilon.\n';

    error_message += ' Matrix 1: [' + m1.join(',') + ']\n';
    error_message += ' Matrix 2: [' + m2.join(',') + ']\n';

    assert_approx_equals(
        m1[mismatched_element], m2[mismatched_element], epsilon,
        error_message);
  }
};

// |m1|, |m2| - arrays of floating point numbers.
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_matrix_significantly_not_equals = function(m1, m2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (m1 == null || m2 == null) {
    assert_not_equals(m1, m2, prefix + "m1 and m2 must not both be null");
    return;
  }

  assert_equals(m1.length, 16, prefix + "m1 must have length of 16");
  assert_equals(m2.length, 16, prefix + "m2 must have length of 16");

  const mismatched_index = get_mismatched_matrix_element_index(m1, m2, epsilon, prefix);
  if (mismatched_index === -1) {
    let error_message = prefix + ' Matrix comparison failed (expected significant difference).\n';
    error_message +=
        ' No element exceeded the given epsilon ' + epsilon + '.\n';

    error_message += ' Matrix 1: [' + m1.join(',') + ']\n';
    error_message += ' Matrix 2: [' + m2.join(',') + ']\n';

    assert_unreached(error_message);
  }
};

// |r1|, |r2| - XRRay objects
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_ray_approx_equals = function(r1, r2, epsilon = FLOAT_EPSILON, prefix = "") {
  assert_point_approx_equals(r1.origin, r2.origin, epsilon, prefix + "origin:");
  assert_point_approx_equals(r1.direction, r2.direction, epsilon, prefix + "direction:");
  assert_matrix_approx_equals(r1.matrix, r2.matrix, epsilon, prefix + "matrix:");
};

// |actualBuffer|, |expectedBuffer| - ArrayBuffer objects
// |message| - string used as a prefix for logging
const assert_array_buffer_equals = function(actualBuffer, expectedBuffer, message = "ArrayBuffers should be equal") {
  if (actualBuffer == null && expectedBuffer == null) {
    return;
  }

  assert_not_equals(actualBuffer, null, message + " (actualBuffer is null)");
  assert_not_equals(expectedBuffer, null, message + " (expectedBuffer is null)");

  assert_equals(actualBuffer.byteLength, expectedBuffer.byteLength, message + " (byteLength mismatch)");

  const actualView = new Uint8Array(actualBuffer);
  const expectedView = new Uint8Array(expectedBuffer);

  for (let i = 0; i < actualView.length; i++) {
    // Check each byte. If a mismatch is found, assert_equals will fail the test
    // and provide a detailed message.
    if (actualView[i] !== expectedView[i]) {
      assert_equals(actualView[i], expectedView[i], `${message} (mismatch at byte ${i})`);
      return;
    }
  }
  // If the loop completes without an assert_equals failure, the buffers are identical.
};

// |actualBuffer|, |expectedBuffer| - ArrayBuffer objects
// |message| - string used as a prefix for logging
const assert_array_buffer_not_equals = function(actualBuffer, expectedBuffer, message = "ArrayBuffers should not be equal") {
  if (actualBuffer == null || expectedBuffer == null) {
    assert_not_equals(actualBuffer, expectedBuffer, message+ " (actualBuffer and expectedBuffer both null)");
    return;
  }

  assert_not_equals(actualBuffer, null, message + " (actualBuffer is null)");
  assert_not_equals(expectedBuffer, null, message + " (expectedBuffer is null)");

  if (actualBuffer.byteLength !== expectedBuffer.byteLength) {
    return;
  }

  const actualView = new Uint8Array(actualBuffer);
  const expectedView = new Uint8Array(expectedBuffer);

  for (let i = 0; i < actualView.length; i++) {
    // Once one byte is different, then the two buffers aren't the same and we
    // can return.
    if (actualView[i] !== expectedView[i]) {
      return;
    }
  }
  assert_unreached(`${message} (buffers are identical`);
};
