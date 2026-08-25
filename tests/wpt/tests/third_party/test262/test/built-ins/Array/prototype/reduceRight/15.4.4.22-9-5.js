// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - callbackfn not called for array with
    one element
---*/

var callCnt = 0;

function callbackfn(prevVal, curVal, idx, obj)
{
  callCnt++;
  return 2;
}

var arr = [1];

assert.sameValue(arr.reduceRight(callbackfn), 1, 'arr.reduceRight(callbackfn)');
assert.sameValue(callCnt, 0, 'callCnt');
