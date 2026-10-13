// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Rejects when getting an enumerable property value throws
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
  ...
  5. For each element key of allKeys, do
    a. Let propertyDesc be ? promises.[[GetOwnProperty]](key).
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      i. Let propertyValue be ? Get(promises, key).
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

var error = new Test262Error();
var input = {};
Object.defineProperty(input, "key", {
  enumerable: true,
  configurable: true,
  get: function() {
    throw error;
  }
});

asyncTest(function() {
  return Promise.allKeyed(input).then(function() {
    throw new Test262Error("The promise should be rejected.");
  }, function(reason) {
    assert.sameValue(reason, error);
  });
});
