// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - modifications to length don't change
    number of iterations
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  return val > 10;
}

var obj = {
  1: 12,
  2: 9,
  length: 2
};

Object.defineProperty(obj, "0", {
  get: function() {
    obj.length = 3;
    return 11;
  },
  configurable: true
});

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert.sameValue(called, 2, 'called');
