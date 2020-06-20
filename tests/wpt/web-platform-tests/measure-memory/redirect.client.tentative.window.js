// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const {iframes, windows} = await build([
    {
      id: 'cross-origin-1',
      redirect: 'client',
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
    {
      id: 'cross-origin-5',
      redirect: 'client',
      window_open: true,
      children: [
        {
          id: 'same-origin-6',
        },
        {
          id: 'cross-origin-7',
        },
        {
          id: 'cross-site-8',
        }
      ],
    },
  ]);
  const keep = sameOriginContexts(frames).concat(sameOriginContexts(windows));
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
}, 'performance.measureMemory does not leak client redirected URL.');
