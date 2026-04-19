// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight not affect call when the array is
    deleted during the call
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  delete o.arr;
  return prevVal + curVal;
}

var o = new Object();
o.arr = ['1', 2, 3, 4, 5];

assert.sameValue(o.arr.reduceRight(callbackfn), "141", 'o.arr.reduceRight(callbackfn)');
assert.sameValue(o.hasOwnProperty("arr"), false, 'o.hasOwnProperty("arr")');
