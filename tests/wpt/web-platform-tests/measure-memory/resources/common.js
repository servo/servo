const SAME_ORIGIN = {origin: get_host_info().HTTPS_ORIGIN, name: "SAME_ORIGIN"};
const CROSS_ORIGIN = {origin: get_host_info().HTTPS_NOTSAMESITE_ORIGIN, name: "CROSS_ORIGIN"}

function checkMeasureMemoryBreakdown(breakdown) {
  assert_own_property(breakdown, 'bytes');
  assert_greater_than_equal(breakdown.bytes, 0);
  assert_own_property(breakdown, 'globals');
  assert_greater_than_equal(breakdown.globals, 0);
  assert_own_property(breakdown, 'type');
  assert_equals(typeof breakdown.type, 'string');
  assert_own_property(breakdown, 'origins');
  assert_greater_than_equal(breakdown.origins.length, 1);
  for (let origin of breakdown.origins) {
    assert_equals(typeof origin, 'string');
  }
}

function checkMeasureMemory(result) {
    assert_own_property(result, 'bytes');
    assert_own_property(result, 'breakdown');
    let bytes = 0;
    for (let breakdown of result.breakdown) {
      checkMeasureMemoryBreakdown(breakdown);
      bytes += breakdown.bytes;
    }
    assert_equals(bytes, result.bytes);
}