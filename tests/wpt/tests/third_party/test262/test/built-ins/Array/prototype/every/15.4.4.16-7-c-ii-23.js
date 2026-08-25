// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - callbackfn called with correct parameters
    (this object O is correct)
---*/

var called = 0;
var obj = {
  0: 11,
  1: 12,
  length: 2
};

function callbackfn(val, idx, o) {
  called++;
  return obj === o;
}

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert.sameValue(called, 2, 'called');
