// META: script=/common/get-host-info.sub.js
// META: script=./resources/checker.js
// META: script=./resources/common.js
// META: timeout=long
'use strict';

function indexOfEmptyEntry(result) {
  return result.breakdown.findIndex(isEmptyBreakdownEntry);
}

promise_test(async testCase => {
  assert_true(self.crossOriginIsolated);

  const initial = await performance.measureUserAgentSpecificMemory();
  let observed_different_order = false;
  for (let i = 0; i < 100; ++i) {
    const current = await performance.measureUserAgentSpecificMemory();
    if (indexOfEmptyEntry(initial) != indexOfEmptyEntry(current)) {
      observed_different_order = true;
    }
  }
  // The order of the breakdown entries must be randomized.
  // A conforming implementation may fail the following assert with
  // the probability of at most 2^-100 since there are at least two
  // entries in the breakdown.
  assert_true(observed_different_order);
}, 'Well-formed result of performance.measureUserAgentSpecificMemory.');
