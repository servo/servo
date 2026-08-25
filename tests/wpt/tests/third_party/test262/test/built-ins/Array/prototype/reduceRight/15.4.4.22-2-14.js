// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight applied to the Array-like object that
    'length' property doesn't exist
---*/

var obj = {
  0: 11,
  1: 12
};
var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return curVal > 10;
}

assert.sameValue(Array.prototype.reduceRight.call(obj, callbackfn, 111), 111, 'Array.prototype.reduceRight.call(obj, callbackfn, 111)');
assert.sameValue(accessed, false, 'accessed');
