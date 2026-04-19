// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allKeyed ignores inherited prototype properties
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
  ...
  6. For each element key of allKeys, do
    a. Let desc be ? promises.[[GetOwnProperty]](key).
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [await-dictionary]
---*/

var proto = { inherited: Promise.resolve('nope') };
var input = Object.create(proto);
input.own = Promise.resolve('yes');

asyncTest(function() {
  return Promise.allKeyed(input).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null);
    assert.compareArray(Object.keys(result), ['own']);
    assert.sameValue(result.own, 'yes');
    assert.sameValue(Object.prototype.hasOwnProperty.call(result, 'inherited'), false);
  });
});
