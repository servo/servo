// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

assert_true(self.crossOriginIsolated);

promise_test(async testCase => {
  const {iframes, windows} = await build([
    {
      id: 'cross-site-1',
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
    checkMeasureMemory(result, [
      {
        url: window.location.href,
        scope: 'Window',
        container: null,
      },
      {
        url: 'cross-origin-url',
        scope: 'cross-origin-aggregated',
        container: {
          id: 'cross-site-1',
          src: iframes['cross-site-1'].src,
        },
      },
      {
        url: windows['same-origin-2'].location.href,
        scope: 'Window',
        container: {
          id: 'cross-site-1',
          src: iframes['cross-site-1'].src,
        },
      },
    ]);
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'performance.measureMemory URLs within a cross-site iframe.');
