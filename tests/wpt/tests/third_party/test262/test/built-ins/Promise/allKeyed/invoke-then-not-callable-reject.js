// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Rejects when promiseResolve returns an object without a callable `then` method
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    ...
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      ...
      iv. Let nextPromise be ? Call(promiseResolve, ctor, « propertyValue »).
      ...
      xiii. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

Promise.resolve = function() {
  return {};
};

asyncTest(function() {
  return assert.throwsAsync(TypeError, function() {
    return Promise.allKeyed({ key: 1 });
  });
});
