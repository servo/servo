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
  const redirecting_child = getUrl(CROSS_ORIGIN, 'resources/redirecting-child.sub.html');
  frame.src = redirecting_child;
  document.body.append(frame);
  await grandchildLoaded;
  try {
    let result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: [window.location.href, redirecting_child]
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'Well-formed result of performance.measureMemory with cross-origin iframe.');
