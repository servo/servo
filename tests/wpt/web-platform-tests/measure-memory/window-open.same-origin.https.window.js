// META: script=/common/get-host-info.sub.js
// META: script=./resources/checker.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  assert_true(self.crossOriginIsolated);

  const {iframes, windows} = await build([
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
  const result = await performance.measureUserAgentSpecificMemory();
  checkMeasureMemory(result, [
    {
      url: window.location.href,
      scope: 'Window',
      container: null,
    },
    {
      url: windows['same-origin-1'].location.href,
      scope: 'Window',
      container: null,
    },
    {
      url: windows['same-origin-2'].location.href,
      scope: 'Window',
      container: null,
    },
    {
      url: windows['same-origin-3'].location.href,
      scope: 'Window',
      container: {
        id: 'same-origin-3',
        src: iframes['same-origin-3'].src,
      },
    },
  ]);
}, 'Well-formed result of performance.measureUserAgentSpecificMemory with same-origin window.open.');
