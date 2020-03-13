// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const grandchildLoaded = new Promise(resolve => {
    window.onmessage = function(message) {
      if (message.data === 'grandchild-loaded') {
        resolve(message);
      }
    }
  });
  const frame = document.createElement('iframe');
  const child = getUrl(SAME_ORIGIN, 'resources/child.sub.html');
  const grandchild = getUrl(SAME_ORIGIN, 'resources/grandchild.sub.html');
  frame.src = child;
  document.body.append(frame);
  await grandchildLoaded;
  try {
    let result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: [window.location.href, child, grandchild],
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'Well-formed result of performance.measureMemory with same-origin iframe.');
