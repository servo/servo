// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Uses the value returned by the constructor's `resolve` method
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  5. For each element key of allKeys, do
    ...
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      ...
      iv. Let nextPromise be ? Call(promiseResolve, ctor, « propertyValue »).
      ...
      xiii. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
features: [await-dictionary]
---*/

var originalCallCount = 0;
var returnedCallCount = 0;

function Constructor(executor) {
  executor(function() {}, function() {});
}
Constructor.resolve = function() {
  return returnedThenable;
};

var originalThenable = {
  then: function() {
    originalCallCount += 1;
  }
};
var returnedThenable = {
  then: function() {
    returnedCallCount += 1;
  }
};

Promise.allSettledKeyed.call(Constructor, { key: originalThenable });

assert.sameValue(originalCallCount, 0, "original thenable not invoked");
assert.sameValue(returnedCallCount, 1, "returned thenable invoked once");
