// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allSettledKeyed resolves to an empty result when all own keys are
  non-enumerable
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
  ...
  5. For each element key of allKeys, do
    a. Let propertyDesc be ? promises.[[GetOwnProperty]](key).
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      ...
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, Reflect]
---*/

var input = {};
Object.defineProperty(input, "first", {
  value: Promise.resolve(1),
  enumerable: false,
  configurable: true
});
Object.defineProperty(input, "second", {
  value: Promise.resolve(2),
  enumerable: false,
  configurable: true
});

asyncTest(function() {
  return Promise.allSettledKeyed(input).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    assert.sameValue(Reflect.ownKeys(result).length, 0, "result has no keys");
  });
});
