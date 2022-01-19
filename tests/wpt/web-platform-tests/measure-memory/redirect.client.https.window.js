// META: script=/common/get-host-info.sub.js
// META: script=./resources/checker.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  assert_true(self.crossOriginIsolated);

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
  const result = await performance.measureUserAgentSpecificMemory();
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
        id: 'cross-origin-1',
        src: frames['cross-origin-1'].src,
      },
    },
    {
      url: windows['same-origin-2'].location.href,
      scope: 'Window',
      container: {
        id: 'cross-origin-1',
        src: iframes['cross-origin-1'].src,
      },
    },
  ]);
}, 'performance.measureUserAgentSpecificMemory does not leak client redirected URL.');
