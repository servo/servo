// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallkeyed
description: >
  Promise.allSettledKeyed includes enumerable symbol-keyed properties and ignores non-enumerable ones
info: |
  PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )

  ...
  1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
  ...
  6. For each element key of allKeys, do
    a. Let desc be ? promises.[[GetOwnProperty]](key).
    b. If desc is not undefined and desc.[[Enumerable]] is true, then
      ...
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, Symbol]
---*/

var sym = Symbol('s');
var hiddenSym = Symbol('hidden');
var input = { str: Promise.resolve(1) };
input[sym] = Promise.resolve(2);
Object.defineProperty(input, hiddenSym, {
  enumerable: false,
  value: Promise.resolve(3)
});

asyncTest(function() {
  return Promise.allSettledKeyed(input).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null);

    var keys = Reflect.ownKeys(result);
    assert.sameValue(keys.length, 2);
    assert.sameValue(keys[0], 'str');
    assert.sameValue(keys[1], sym);

    assert.sameValue(result.str.status, 'fulfilled');
    assert.sameValue(result.str.value, 1);
    assert.sameValue(result[sym].status, 'fulfilled');
    assert.sameValue(result[sym].value, 2);
    assert.sameValue(Object.prototype.hasOwnProperty.call(result, hiddenSym), false);
  });
});
