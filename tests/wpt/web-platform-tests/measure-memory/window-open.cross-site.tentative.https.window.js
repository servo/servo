// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

assert_true(self.crossOriginIsolated);

promise_test(async testCase => {
  const {iframes, windows} = await build([
    {
      id: 'cross-site-1',
      window_open: true,
      children: [
        {
          id: 'same-origin-2',
          window_open: true,
        },
        {
          id: 'same-origin-3',
        },
        {
          id: 'cross-origin-4',
        },
      ]
    },
  ]);
  const result = await performance.measureMemory();
  checkMeasureMemory(result, [
    {
      url: window.location.href,
      scope: 'Window',
      container: null,
    },
  ]);
}, 'performance.measureMemory does not leak URL of cross-site window.open.');
