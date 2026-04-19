// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - Date Object can be used as
    accumulator
---*/

var accessed = false;
var objDate = new Date(0);

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return prevVal === objDate;
}

var obj = {
  0: 11,
  length: 1
};

assert.sameValue(Array.prototype.reduceRight.call(obj, callbackfn, objDate), true, 'Array.prototype.reduceRight.call(obj, callbackfn, objDate)');
assert(accessed, 'accessed !== true');
