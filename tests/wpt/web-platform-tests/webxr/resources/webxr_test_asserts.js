// Utility assert functions.
// Relies on resources/testharness.js to be included before this file.
// Relies on webxr_test_constants.js to be included before this file.

// |p1|, |p2| - objects with x, y, z, w components that are floating point numbers
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_point_approx_equals = function(p1, p2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (p1 == null && p2 == null) {
    return;
  }

  assert_not_equals(p1, null, prefix + "p1 must be non-null");
  assert_not_equals(p2, null, prefix + "p2 must be non-null");

  let mismatched_component = null;
  for (const v of ['x', 'y', 'z', 'w']) {
    if (Math.abs(p1[v] - p2[v]) > epsilon) {
      mismatched_component = v;
      break;
    }
  }

  if (mismatched_component !== null) {
    let error_message = prefix + ' Point comparison failed.\n';
    error_message += ` p1: {x: ${p1.x}, y: ${p1.y}, z: ${p1.z}, w: ${p1.w}}\n`;
    error_message += ` p2: {x: ${p2.x}, y: ${p2.y}, z: ${p2.z}, w: ${p2.w}}\n`;
    error_message += ` Difference in component ${mismatched_component} exceeded the given epsilon.\n`;
    assert_approx_equals(p2[mismatched_component], p1[mismatched_component], epsilon, error_message);
  }
};

// |p1|, |p2| - objects with x, y, z, w components that are floating point numbers
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_point_significantly_not_equals = function(p1, p2, epsilon = FLOAT_EPSILON, prefix = "") {

  assert_not_equals(p1, null, prefix + "p1 must be non-null");
  assert_not_equals(p2, null, prefix + "p2 must be non-null");

  let mismatched_component = null;
  for (const v of ['x', 'y', 'z', 'w']) {
    if (Math.abs(p1[v] - p2[v]) > epsilon) {
      mismatched_component = v;
      break;
    }
  }

  if (mismatched_component === null) {
    let error_message = prefix + ' Point comparison failed.\n';
    error_message += ` p1: {x: ${p1.x}, y: ${p1.y}, z: ${p1.z}, w: ${p1.w}}\n`;
    error_message += ` p2: {x: ${p2.x}, y: ${p2.y}, z: ${p2.z}, w: ${p2.w}}\n`;
    error_message += ` Difference in components did not exceeded the given epsilon.\n`;
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
  assert_point_approx_equals(t1.orientation, t2.orientation, epsilon, prefix + "orientations must be equal");
};

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

  let mismatched_element = -1;
  for (let i = 0; i < 16; ++i) {
    if (Math.abs(m1[i] - m2[i]) > epsilon) {
      mismatched_element = i;
      break;
    }
  }

  if (mismatched_element > -1) {
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

// |m1|, |m2| - arrays of floating point numbers
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
const assert_matrix_significantly_not_equals = function(m1, m2, epsilon = FLOAT_EPSILON, prefix = "") {
  if (m1 == null && m2 == null) {
    return;
  }

  assert_not_equals(m1, null, prefix + "m1 must be non-null");
  assert_not_equals(m2, null, prefix + "m2 must be non-null");

  assert_equals(m1.length, 16, prefix + "m1 must have length of 16");
  assert_equals(m2.length, 16, prefix + "m2 must have length of 16");

  let mismatch = false;
  for (let i = 0; i < 16; ++i) {
    if (Math.abs(m1[i] - m2[i]) > epsilon) {
      mismatch = true;
      break;
    }
  }

  if (!mismatch) {
    let m1_str = '[';
    let m2_str = '[';
    for (let i = 0; i < 16; ++i) {
      m1_str += m1[i] + (i < 15 ? ', ' : '');
      m2_str += m2[i] + (i < 15 ? ', ' : '');
    }
    m1_str += ']';
    m2_str += ']';

    let error_message = prefix + 'Matrix comparison failed.\n';
    error_message +=
        ' No element exceeded the given epsilon ' + epsilon + '.\n';

    error_message += ' Matrix A: ' + m1_str + '\n';
    error_message += ' Matrix B: ' + m2_str + '\n';

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
