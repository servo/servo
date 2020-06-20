// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const {iframes} = await build([
    {
      id: 'cross-origin-1',
      children: [
        {
          id: 'same-origin-2',
        },
        {
          id: 'cross-origin-3',
        },
        {
          id: 'cross-site-4',
        }
      ],
    },
  ]);
  try {
    const result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: [
        window.location.href,
        iframes['cross-origin-1'].src,
      ],
      required: [
        window.location.href,
      ],
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'performance.measureMemory URLs within a cross-origin iframe.');
