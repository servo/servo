// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const {windows, iframes} = await build([
    {
      id: 'same-origin-1',
      window_open: true,
      children: [
        {
          id: 'same-origin-2',
          window_open: true,
        },
        {
          id: 'same-origin-3',
        },
      ]
    },
  ]);
  try {
    const result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: [
        window.location.href,
        windows['same-origin-1'].location.href,
        windows['same-origin-2'].location.href,
        iframes['same-origin-3'].src,
      ],
      required: [
        window.location.href,
        windows['same-origin-1'].location.href,
        windows['same-origin-2'].location.href,
        iframes['same-origin-3'].src,
      ],
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'Well-formed result of performance.measureMemory with same-origin window.open.');
