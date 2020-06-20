// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const {iframes} = await build([
    {
      id: 'same-origin-1',
      children: [
        {
          id: 'same-origin-2',
        }
      ],
    },
  ]);
  try {
    const result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: [
        window.location.href,
        iframes['same-origin-1'].src,
        iframes['same-origin-2'].src,
      ],
      required: [
        window.location.href,
        iframes['same-origin-1'].src,
        iframes['same-origin-2'].src,
      ]
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'Well-formed result of performance.measureMemory with same-origin iframes.');
