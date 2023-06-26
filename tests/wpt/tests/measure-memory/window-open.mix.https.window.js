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
      children: [
        {
          id: 'same-origin-2',
          window_open: true,
          children: [
            {
              id: 'same-origin-3',
              window_open: true,
            },
          ],
        },
        {
          id: 'cross-origin-4',
          children: [
            {
              id: 'same-origin-5',
              window_open: true,
            },
          ],
        },
        {
          id: 'cross-site-6',
          children: [
            {
              id: 'same-origin-7',
              window_open: true,
            },
          ],
        },
        {
          id: 'same-origin-8',
          children: [
            {
              id: 'cross-origin-9',
              window_open: true,
              children: [
                {
                  id: 'same-origin-10',
                },
                {
                  id: 'same-origin-11',
                  window_open: true,
                },
              ],
            },
            {
              id: 'cross-site-12',
              window_open: true,
              children: [
                {
                  id: 'same-origin-13',
                },
                {
                  id: 'same-origin-14',
                  window_open: true,
                },
              ],
            },
          ],
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
      container: {
        id: 'same-origin-1',
        src: iframes['same-origin-1'].src,
      },
    },
    {
      url: windows['same-origin-2'].location.href,
      scope: 'Window',
      container: null,
    },
    {
      url: windows['same-origin-3'].location.href,
      scope: 'Window',
      container: null,
    },
    {
      url: 'cross-origin-url',
      scope: 'cross-origin-aggregated',
      container: {
        id: 'cross-origin-4',
        src: iframes['cross-origin-4'].src,
      },
    },
    {
      url: windows['same-origin-5'].location.href,
      scope: 'Window',
      container: null,
    },
    {
      url: 'cross-origin-url',
      scope: 'cross-origin-aggregated',
      container: {
        id: 'cross-site-6',
        src: iframes['cross-site-6'].src,
      },
    },
    {
      url: windows['same-origin-7'].location.href,
      scope: 'Window',
      container: null,
    },
    {
      url: windows['same-origin-8'].location.href,
      scope: 'Window',
      container: {
        id: 'same-origin-8',
        src: iframes['same-origin-8'].src,
      },
    },
  ]);
}, 'performance.measureUserAgentSpecificMemory does not leak URLs in cross-origin iframes and windows.');
