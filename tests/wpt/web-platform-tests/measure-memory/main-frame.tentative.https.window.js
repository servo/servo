// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

assert_true(self.crossOriginIsolated);
promise_test(async testCase => {
  const result = await performance.measureMemory();
  checkMeasureMemory(result, [
    {
      url: window.location.href,
      scope: 'Window',
      container: null,
    },
  ]);
}, 'Well-formed result of performance.measureMemory.');
