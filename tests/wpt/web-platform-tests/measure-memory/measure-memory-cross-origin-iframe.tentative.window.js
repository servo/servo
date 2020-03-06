// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const frame = document.createElement("iframe");
  const child = getUrl(CROSS_ORIGIN, "resources/child.sub.html");
  const grandchild = getUrl(CROSS_ORIGIN, "resources/grandchild.sub.html");
  frame.src = child;
  document.body.append(frame);
  try {
    let result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: [window.location.href, child]
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'Well-formed result of performance.measureMemory with cross-origin iframe.');
