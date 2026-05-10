// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some applied to String object which implements its
    own property get method
---*/

function callbackfn1(val, idx, obj) {
  return parseInt(val, 10) > 1;
}

function callbackfn2(val, idx, obj) {
  return parseInt(val, 10) > 2;
}

var str = new String("12");

String.prototype[2] = "3";

assert(Array.prototype.some.call(str, callbackfn1), 'Array.prototype.some.call(str, callbackfn1) !== true');
assert.sameValue(Array.prototype.some.call(str, callbackfn2), false, 'Array.prototype.some.call(str, callbackfn2)');
