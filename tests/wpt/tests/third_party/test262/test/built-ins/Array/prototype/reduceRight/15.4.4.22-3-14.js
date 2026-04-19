// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - value of 'length' is a string
    containing -Infinity
---*/

var accessed2 = false;

function callbackfn2(prevVal, curVal, idx, obj) {
  accessed2 = true;
}

var obj2 = {
  0: 9,
  length: "-Infinity"
};

assert.sameValue(Array.prototype.reduceRight.call(obj2, callbackfn2, 2), 2, 'Array.prototype.reduceRight.call(obj2, callbackfn2, 2)');
assert.sameValue(accessed2, false, 'accessed2');
