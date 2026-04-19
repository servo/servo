// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - value of 'length' is a string
    containing a negative number
---*/

var testResult1 = true;
var testResult2 = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx > 1) {
    testResult1 = false;
  }

  if (idx === 1) {
    testResult2 = true;
  }
  return false;
}

var obj = {
  0: 11,
  1: 12,
  2: 9,
  length: "-4294967294"
};

Array.prototype.reduceRight.call(obj, callbackfn, 1);

assert(testResult1, 'testResult1 !== true');
assert.sameValue(testResult2, false, 'testResult2');
