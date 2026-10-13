// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Error thrown when accessing the instance's `then` method (rejecting Promise)
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    ...
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
      iv. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
      ...
      xiii. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

var error = new Test262Error();
var promise = new Promise(function() {});

Object.defineProperty(promise, "then", {
  get: function() {
    throw error;
  }
});

asyncTest(function() {
  return Promise.allSettledKeyed({ key: promise }).then(function() {
    throw new Test262Error('The promise should be rejected.');
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
