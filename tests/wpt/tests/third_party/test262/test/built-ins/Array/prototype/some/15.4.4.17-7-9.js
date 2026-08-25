// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - modifications to length don't change number
    of iterations
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  return val > 10;
}

var obj = {
  0: 9,
  2: 12,
  length: 3
};

Object.defineProperty(obj, "1", {
  get: function() {
    obj.length = 2;
    return 8;
  },
  configurable: true
});

assert(Array.prototype.some.call(obj, callbackfn), 'Array.prototype.some.call(obj, callbackfn) !== true');
assert.sameValue(called, 3, 'called');
