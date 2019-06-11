// pieced together from various things in wpt/webxr/resources

const VALID_PROJECTION_MATRIX = [1, 0, 0, 0, 0, 1, 0, 0, 3, 2, -1, -1, 0, 0, -0.2, 0];
const LEFT_OFFSET = {position: [-0.1, 0, 0], orientation: [0,0,0,0]};
const RIGHT_OFFSET = {position: [0.1, 0, 0], orientation: [0,0,0,0]};

let assert_matrix_approx_equals = function(m1, m2, epsilon, prefix = "") {
  assert_equals(m1.length, m2.length, prefix + "Matrix lengths should match");
  for(var i = 0; i < m1.length; ++i) {
    assert_approx_equals(m1[i], m2[i], epsilon, m1 + prefix + "Component number " + i + " should match");
  }
}

const TEST_VIEWS = [
    {eye: "left", projectionMatrix: VALID_PROJECTION_MATRIX, viewOffset: LEFT_OFFSET},
    {eye: "right", projectionMatrix: VALID_PROJECTION_MATRIX, viewOffset: RIGHT_OFFSET}
];
