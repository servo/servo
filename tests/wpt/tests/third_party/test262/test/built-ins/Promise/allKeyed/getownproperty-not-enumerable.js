// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Keys whose property descriptor has [[Enumerable]] false are skipped
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    a. Let desc be ? promises.[[GetOwnProperty]](key).
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, Reflect]
---*/

var input = Object.create(null);
Object.defineProperty(input, "a", {
  value: Promise.resolve(1),
  enumerable: true,
  configurable: true
});
Object.defineProperty(input, "b", {
  value: Promise.resolve(2),
  enumerable: false,
  configurable: true
});
Object.defineProperty(input, "c", {
  value: Promise.resolve(3),
  enumerable: true,
  configurable: true
});

asyncTest(function() {
  return Promise.allKeyed(input).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    var keys = Reflect.ownKeys(result);
    assert.sameValue(keys.length, 2, "only enumerable keys are present");
    assert.sameValue(keys[0], "a", "first key");
    assert.sameValue(keys[1], "c", "second key");
    assert.sameValue(result.a, 1, "result.a");
    assert.sameValue(result.c, 3, "result.c");
  });
});
