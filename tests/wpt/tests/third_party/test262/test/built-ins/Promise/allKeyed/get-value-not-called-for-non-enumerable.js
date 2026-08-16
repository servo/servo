// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Does not get values for non-enumerable own properties
info: |
  PerformPromiseAllKeyed ( variant, promises, ctor, resultCapability, promiseResolve )

  ...
  5. For each element key of allKeys, do
    a. Let propertyDesc be ? promises.[[GetOwnProperty]](key).
    b. If propertyDesc is not undefined and propertyDesc.[[Enumerable]] is true, then
      i. Let propertyValue be ? Get(promises, key).
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

var input = {
  visible: Promise.resolve(1)
};
Object.defineProperty(input, "hidden", {
  enumerable: false,
  configurable: true,
  get: function() {
    throw new Test262Error("non-enumerable getter should not be called");
  }
});

asyncTest(function() {
  return Promise.allKeyed(input).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    assert.compareArray(Object.keys(result), ["visible"]);
    assert.sameValue(result.visible, 1);
    assert.sameValue(Object.prototype.hasOwnProperty.call(result, "hidden"), false);
  });
});
