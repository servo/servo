// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  Rejects when resolving the final result throws
info: |
  Promise.allSettledKeyed ( promises )

  ...
  6. Let result be Completion(PerformPromiseAllKeyed(...)).
  7. IfAbruptRejectPromise(result, promiseCapability).

  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
  If remainingElementsCount.[[Value]] = 0, then
    Let result be CreateKeyedPromiseCombinatorResultObject(entries).
    Perform ? Call(resultCapability.[[Resolve]], undefined, « result »).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

var error = new Test262Error();
function Constructor(executor) {
  return new Promise(function(_, reject) {
    executor(function() {
      throw error;
    }, reject);
  });
}
Constructor.resolve = Promise.resolve;

asyncTest(function() {
  return Promise.allSettledKeyed.call(Constructor, {}).then(function() {
    throw new Test262Error("The promise should be rejected.");
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
