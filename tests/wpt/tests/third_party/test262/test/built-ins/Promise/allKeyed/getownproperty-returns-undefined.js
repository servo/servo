// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Keys where [[GetOwnProperty]] returns undefined are skipped
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  6. For each element key of allKeys, do
    a. Let desc be ? promises.[[GetOwnProperty]](key).
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, Proxy, Reflect]
---*/

var input = new Proxy({ key: Promise.resolve(1) }, {
  getOwnPropertyDescriptor: function() {
    return undefined;
  }
});

asyncTest(function() {
  return Promise.allKeyed(input).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null, "result is null-prototype");
    assert.sameValue(Reflect.ownKeys(result).length, 0, "no keys in result");
  });
});
