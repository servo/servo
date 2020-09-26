// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';


promise_test(async testCase => {
 try {
    const BYTES_PER_WORKER = 10 * 1024 * 1024;
    await createWorker(BYTES_PER_WORKER);
    const result = await performance.measureMemory();
    assert_greater_than_equal(result.bytes, BYTES_PER_WORKER);
    checkMeasureMemory(result, {
      allowed: [window.location.href],
      required: [window.location.href],
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'Well-formed result of performance.measureMemory.');

