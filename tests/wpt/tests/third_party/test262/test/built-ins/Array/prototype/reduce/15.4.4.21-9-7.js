// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce stops calling callbackfn once the array is
    deleted during the call
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  delete o.arr;
  return prevVal + curVal;
}

var o = new Object();
o.arr = ['1', 2, 3, 4, 5];

assert.sameValue(o.arr.reduce(callbackfn), "12345", 'o.arr.reduce(callbackfn)');
assert.sameValue(o.hasOwnProperty("arr"), false, 'o.hasOwnProperty("arr")');
