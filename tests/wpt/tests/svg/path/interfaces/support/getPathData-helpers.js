// Asserts that |actual| is a sequence of `SVGPathSegment`-shaped objects
// equivalent to |expected|. Numeric values are compared with
// `assert_array_approx_equals` and |epsilon| (default 0, i.e. exact).
function assert_path_data_equals(actual, expected, epsilon = 0) {
  assert_equals(actual.length, expected.length, "segment count");
  for (let i = 0; i < expected.length; ++i) {
    assert_equals(actual[i].type, expected[i].type, `segment[${i}].type`);
    assert_array_approx_equals(
        actual[i].values, expected[i].values, epsilon,
        `segment[${i}].values`);
  }
}

// Creates a fresh detached <path> (optionally with a starting 'd').
function createPath(d) {
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  if (d !== undefined) {
    path.setAttribute("d", d);
  }
  return path;
}
