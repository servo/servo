function checkMeasureMemoryResultEntry(entry, checkUrl) {
  assert_own_property(entry, "jsMemoryEstimate");
  assert_own_property(entry, "jsMemoryRange");
  assert_equals(entry.jsMemoryRange.length, 2);
  assert_greater_than_equal(entry.jsMemoryRange[1], entry.jsMemoryRange[0]);
  assert_greater_than_equal(entry.jsMemoryEstimate, entry.jsMemoryRange[0]);
  assert_greater_than_equal(entry.jsMemoryRange[1], entry.jsMemoryEstimate);
  if (checkUrl) {
    assert_own_property(entry, "url");
  }
}

function checkMeasureMemoryResultSummary(result) {
    assert_own_property(result, "total");
    checkMeasureMemoryResultEntry(result.total, false);
}

function checkMeasureMemoryResultDetails(result) {
    assert_own_property(result, "current");
    checkMeasureMemoryResultEntry(result.current, true);
    assert_own_property(result, "other");
    for (other of result.other) {
      checkMeasureMemoryResultEntry(other, true);
    }
}

promise_test(async testCase => {
  try {
    let result = await performance.measureMemory();
    checkMeasureMemoryResultSummary(result);
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, "SecurityError");
  }
}, 'Well-formed result of performance.measureMemory with default arguments.');

promise_test(async testcase => {
  try {
    let result = await performance.measureMemory({detailed: false});
    checkMeasureMemoryResultSummary(result);
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, "SecurityError");
  }
}, 'well-formed result of performance.measureMemory with detailed=false.');

promise_test(async testcase => {
  try {
    let result = await performance.measureMemory({detailed: true});
    checkMeasureMemoryResultSummary(result);
    checkMeasureMemoryResultDetails(result);
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, "SecurityError");
  }
}, 'well-formed result of performance.measureMemory with detailed=true.');
