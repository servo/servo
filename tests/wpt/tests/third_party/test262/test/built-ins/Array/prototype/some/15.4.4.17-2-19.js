// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some applied to Function object which implements
    its own property get method
---*/

function callbackfn1(val, idx, obj) {
  return val > 10;
}

function callbackfn2(val, idx, obj) {
  return val > 11;
}

var fun = function(a, b) {
  return a + b;
};
fun[0] = 9;
fun[1] = 11;
fun[2] = 12;

assert(Array.prototype.some.call(fun, callbackfn1), 'Array.prototype.some.call(fun, callbackfn1) !== true');
assert.sameValue(Array.prototype.some.call(fun, callbackfn2), false, 'Array.prototype.some.call(fun, callbackfn2)');
