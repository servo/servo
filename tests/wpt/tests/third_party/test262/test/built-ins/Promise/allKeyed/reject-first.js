// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allKeyed rejects when the first promise to settle is rejected
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    ...
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      ...
      8. If variant is all, then
        a. Let onRejected be resultCapability.[[Reject]].
      ...
      11. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

var error = new Test262Error();
var rejectFirst, resolveSecond, resolveThird;

var input = {
  first: new Promise(function(_, reject) { rejectFirst = reject; }),
  second: new Promise(function(resolve) { resolveSecond = resolve; }),
  third: new Promise(function(resolve) { resolveThird = resolve; })
};

var combined = Promise.allKeyed(input);

rejectFirst(error);
resolveSecond("b");
resolveThird("c");

asyncTest(function() {
  return combined.then(function() {
    throw new Test262Error("The promise should not be fulfilled.");
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
