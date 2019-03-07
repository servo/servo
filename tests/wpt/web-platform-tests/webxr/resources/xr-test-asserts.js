// Utility assert functions.
// Relies on resources/testharness.js to be included before this file.

// |p1|, |p2| - objects with x, y, z, w components that are floating point numbers
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
let assert_point_approx_equals = function(p1, p2, epsilon, prefix = "") {
  assert_approx_equals(p1.x, p2.x, epsilon, prefix + "xs must match");
  assert_approx_equals(p1.y, p2.y, epsilon, prefix + "ys must match");
  assert_approx_equals(p1.z, p2.z, epsilon, prefix + "zs must match");
  assert_approx_equals(p1.w, p2.w, epsilon, prefix + "ws must match");
};

// |m1|, |m2| - arrays of floating point numbers
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
let assert_matrix_approx_equals = function(m1, m2, epsilon, prefix = "") {
  assert_equals(m1.length, m2.length, prefix + "Matrix lengths should match");
  for(var i = 0; i < m1.length; ++i) {
    assert_approx_equals(m1[i], m2[i], epsilon, prefix + "Component number " + i + " should match");
  }
}

// |r1|, |r2| - XRRay objects
// |epsilon| - float specifying precision
// |prefix| - string used as a prefix for logging
let assert_ray_approx_equals = function(r1, r2, epsilon, prefix = "") {
  assert_point_approx_equals(r1.origin, r2.origin, epsilon, prefix + "origin:");
  assert_point_approx_equals(r1.direction, r2.direction, epsilon, prefix + "direction:");
  assert_matrix_approx_equals(r1.matrix, r2.matrix, epsilon, prefix + "matrix:");
}
