// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allKeyed ignores non-enumerable own properties
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    a. Let desc be ? promises.[[GetOwnProperty]](key).
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

var input = {
  visible: Promise.resolve(2)
};

Object.defineProperty(input, 'hidden', {
  enumerable: false,
  value: Promise.resolve(1)
});

asyncTest(function() {
  return Promise.allKeyed(input).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null);
    assert.compareArray(Object.keys(result), ['visible']);
    assert.sameValue(result.visible, 2);
    assert.sameValue(Object.prototype.hasOwnProperty.call(result, 'hidden'), false);
  });
});
