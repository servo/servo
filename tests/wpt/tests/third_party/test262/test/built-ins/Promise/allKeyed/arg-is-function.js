// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allkeyed
description: >
  Promise.allKeyed accepts a function argument with enumerable own properties
info: |
  Promise.allKeyed ( promises )

  ...
  5. If promises is not an Object, then
    a. ...Reject...
  ...

  Functions are objects, so they pass the type check. Only own enumerable
  properties are traversed; built-in function properties (name, length,
  prototype) are non-enumerable by default and should be excluded.
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary]
---*/

function fn() {}
fn.key = Promise.resolve('val');

asyncTest(function() {
  return Promise.allKeyed(fn).then(function(result) {
    assert.sameValue(Object.getPrototypeOf(result), null);

    var keys = Reflect.ownKeys(result);
    assert.sameValue(keys.length, 1);
    assert.sameValue(keys[0], 'key');
    assert.sameValue(result.key, 'val');

    assert.sameValue(Object.prototype.hasOwnProperty.call(result, 'name'), false);
    assert.sameValue(Object.prototype.hasOwnProperty.call(result, 'length'), false);
    assert.sameValue(Object.prototype.hasOwnProperty.call(result, 'prototype'), false);
  });
});
