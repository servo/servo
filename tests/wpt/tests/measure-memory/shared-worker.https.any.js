// META: script=./resources/checker.js
// META: global=sharedworker
'use strict';

promise_test(async testCase => {
  assert_true(self.crossOriginIsolated);

  const result = await performance.measureUserAgentSpecificMemory();

  checkMeasureMemory(result, [
    {
      url: self.location.href,
      scope: 'SharedWorkerGlobalScope',
      container: null,
    },
  ]);
}, 'Well-formed result of performance.measureUserAgentSpecificMemory.');

