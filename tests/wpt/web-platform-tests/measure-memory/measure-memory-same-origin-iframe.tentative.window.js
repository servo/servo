// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const frame = document.createElement("iframe");
  const path = new URL("resources/iframe.sub.html", window.location).pathname;
  frame.src = `${SAME_ORIGIN}${path}`;
  document.body.append(frame);
  try {
    let result = await performance.measureMemory();
    checkMeasureMemory(result);
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'Well-formed result of performance.measureMemory with same-origin iframe.');
