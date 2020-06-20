// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const {windows, iframes} = await build([
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
  try {
    const result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: [
        window.location.href,
        iframes['same-origin-1'].src,
        windows['same-origin-2'].location.href,
        windows['same-origin-3'].location.href,
        iframes['cross-origin-4'].src,
        iframes['cross-site-6'].src,
        iframes['same-origin-8'].src,
      ],
      required: [
        window.location.href,
        iframes['same-origin-1'].src,
        windows['same-origin-2'].location.href,
        windows['same-origin-3'].location.href,
        iframes['same-origin-8'].src,
      ],
    });
  } catch (error) {
    if (!(error instanceof DOMException)) {
      throw error;
    }
    assert_equals(error.name, 'SecurityError');
  }
}, 'performance.measureMemory does not leak URLs in cross-origin iframes and windows.');
