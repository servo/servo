// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
function f(array, method, args) {
  var called = false;
  var keys = [];
  for (var key in array) {
    keys.push(key);
    if (!called) {
      called = true;
      Reflect.apply(method, array, args);
    }
  }
  return keys;
}

assert.compareArray(f([1, /* hole */, 3], Array.prototype.unshift, [0]), ["0"]);
assert.compareArray(f([1, /* hole */, 3], Array.prototype.splice, [0, 0, 0]), ["0"]);

