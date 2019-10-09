function checkMeasureMemoryResultSummary(result) {
    assert_own_property(result, "total");
    assert_own_property(result.total, "jsMemoryEstimate");
    assert_own_property(result.total, "jsMemoryRange");
    assert_equals(result.total.jsMemoryRange.length, 2);
    assert_greater_than_equal(
        result.total.jsMemoryRange[1],
        result.total.jsMemoryRange[0]);
    assert_greater_than_equal(
        result.total.jsMemoryEstimate,
        result.total.jsMemoryRange[0]);
    assert_greater_than_equal(
        result.total.jsMemoryRange[1],
        result.total.jsMemoryEstimate);
}

promise_test(async testCase => {
  let result = await performance.measureMemory();
  checkMeasureMemoryResultSummary(result);
}, 'Well-formed result of performance.measureMemory with default arguments.');

promise_test(async testcase => {
  let result = await performance.measureMemory({detailed: false});
  checkMeasureMemoryResultSummary(result);
}, 'well-formed result of performance.measurememory with detailed=false.');
