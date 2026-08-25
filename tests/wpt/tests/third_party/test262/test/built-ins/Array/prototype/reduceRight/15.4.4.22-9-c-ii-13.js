// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - callbackfn is called with 4 formal
    parameter
---*/

var arr = [11, 12, 13];
var initVal = 6.99;
var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 2) {
    testResult = (prevVal === initVal);
  }
  return curVal > 10 && obj[idx] === curVal;
}

assert.sameValue(arr.reduceRight(callbackfn, initVal), true, 'arr.reduceRight(callbackfn, initVal)');
assert(testResult, 'testResult !== true');
