// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-safe-perform-promise-all
description: >
  `import.defer` with transitive async dependencies does not call Promise.prototype.then
info: |
  SafePerformPromiseAll ( _promises_ )
    1. Let _resultCapability_ be ! NewPromiseCapability(%Promise%).
    1. If _promises_ is empty, then
      1. Perform ! Call(_resultCapability_.[[Resolve]], *undefined*, &laquo; CreateArrayFromList(&laquo; &raquo;) &raquo;).
      1. Return _resultCapability_.[[Promise]].
    1. Let _values_ be a new empty List.
    1. Let _remainingElementsCount_ be the Record { [[Value]]: the number of elements in _promises_ }.
    1. Let _index_ be 0.
    1. For each element _promise_ of _promises_, do
      1. Append *undefined* to _values_.
      1. Let _onFulfilled_ be CreatePromiseAllResolveElement(_index_, _values_, _resultCapability_, _remainingElementsCount_).
      1. Set _index_ to _index_ + 1.
      1. Perform PerformPromiseThen(_promise_, _onFulfilled_, _resultCapability_.[[Reject]]).
    1. Return _resultCapability_.[[Promise]].

  SafePerformPromiseAll calls PerformPromiseThen directly on each
  evaluation promise, rather than looking up "then" on the promise.
  This means that patching Promise.prototype.then must not affect the
  internal promise aggregation.

flags: [module, async]
features: [import-defer, top-level-await]
includes: [compareArray.js]
---*/

import "./setup_FIXTURE.js";

var thenCallCount = 0;
var originalThen = Promise.prototype.then;

Promise.prototype.then = function(onFulfilled, onRejected) {
  thenCallCount++;
  return originalThen.call(this, onFulfilled, onRejected);
};

thenCallCount = 0;

var p = import.defer("./imports-tla_FIXTURE.js");

// At this point we use `originalThen` to avoid polluting `thenCallCount`.
originalThen.call(
  originalThen.call(p, ns => {
    Promise.prototype.then = originalThen;

    assert.sameValue(thenCallCount, 0, "Promise.prototype.then must not be called by import.defer internals");
    assert.compareArray(globalThis.evaluations, ["tla start", "tla end"]);
    ns.x;
    assert.compareArray(globalThis.evaluations, ["tla start", "tla end", "imports-tla"]);
  }),
  $DONE,
  $DONE
);
