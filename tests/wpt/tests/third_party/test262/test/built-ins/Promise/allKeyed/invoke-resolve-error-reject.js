// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Calling promiseResolve throws, resulting in promise rejection
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    ...
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
      iv. Let nextPromise be ? Call(promiseResolve, constructor, « value »).

  Promise.allKeyed ( promises )

  ...
  6. Let result be Completion(PerformPromiseAllKeyed(...)).
  7. IfAbruptRejectPromise(result, promiseCapability).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

var error = new Test262Error();
Promise.resolve = function() {
  throw error;
};

asyncTest(function() {
  return Promise.allKeyed({ key: 1 }).then(function() {
    throw new Test262Error('The promise should be rejected.');
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
