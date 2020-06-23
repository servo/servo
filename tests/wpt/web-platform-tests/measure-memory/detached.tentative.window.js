// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const {iframes, windows} = await build([
    {
      id: 'cross-site-1',
      children: [
        {
          id: 'same-origin-2',
        },
        {
          id: 'same-origin-11',
          window_open: true,
        },
      ],
    },
    {
      id: 'same-origin-3',
      children: [
        {
          id: 'same-origin-4',
        },
        {
          id: 'same-origin-12',
          window_open: true,
        },
      ],
    },
    {
      id: 'cross-origin-5',
      children: [
        {
          id: 'same-origin-6',
        },
        {
          id: 'same-origin-13',
          window_open: true,
        },
      ],
    },
    {
      id: 'same-origin-7',
      window_open: true,
      children: [
        {
          id: 'same-origin-8',
        }
      ],
    },
    {
      id: 'cross-origin-9',
      window_open: true,
      children: [
        {
          id: 'same-origin-10',
        }
      ],
    },
  ]);
  const allowed = [
    window.location.href,
    iframes['cross-site-1'].src,
    iframes['same-origin-3'].src,
    iframes['same-origin-4'].src,
    iframes['cross-origin-5'].src,
    iframes['same-origin-8'].sec,
    windows['same-origin-7'].location.href,
    windows['same-origin-12'].location.href,
  ];
  const keep = sameOriginContexts(frames).concat(sameOriginContexts(windows));
  // Detach iframes:
  // 1) By setting src attribute:
  iframes['cross-site-1'].src =
      iframes['cross-site-1'].src.replace('iframe.sub', 'iframe.secret.sub');
  // 2) By setting location attribute:
  let url = iframes['same-origin-3'].contentWindow.location.href;
  url = url.replace('iframe.sub', 'iframe.secret.sub');
  iframes['same-origin-3'].contentWindow.location.href = url;
  // 3) By removing from the DOM tree:
  iframes['cross-origin-5'].parentNode.removeChild(iframes['cross-origin-5']);

  // Detach windows:
  // 1) By setting document.location attribute:
  url = windows['same-origin-7'].location.href;
  url = url.replace('window.sub', 'window.secret.sub');
  windows['same-origin-7'].location.href = url;
  // 2) By closing the window:
  windows['cross-origin-9'].close();

  await waitForMessage('cross-site-1');
  await waitForMessage('same-origin-3');
  await waitForMessage('same-origin-7');

  try {
    const result = await performance.measureMemory();
    checkMeasureMemory(result, {
      allowed: allowed.concat([
        iframes['cross-site-1'].src,
        iframes['same-origin-3'].contentWindow.location.href,
        windows['same-origin-7'].location.href,
      ]),
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
}, 'performance.measureMemory URLs within a cross-site iframe.');
