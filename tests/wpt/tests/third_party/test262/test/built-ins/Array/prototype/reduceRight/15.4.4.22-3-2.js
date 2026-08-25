// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight applied to an Array-like object,
    'length' is 0 (length overridden to false(type conversion))
---*/

var accessed = false;

function callbackfn(preVal, curVal, idx, obj) {
  accessed = true;
}

var obj = {
  0: 9,
  length: false
};

assert.sameValue(Array.prototype.reduceRight.call(obj, callbackfn, 1), 1, 'Array.prototype.reduceRight.call(obj, callbackfn, 1)');
assert.sameValue(accessed, false, 'accessed');
