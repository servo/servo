// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - the Math Object can be used as
    accumulator
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return prevVal === Math;
}

var obj = {
  0: 11,
  length: 1
};

assert.sameValue(Array.prototype.reduceRight.call(obj, callbackfn, Math), true, 'Array.prototype.reduceRight.call(obj, callbackfn, Math)');
assert(accessed, 'accessed !== true');
