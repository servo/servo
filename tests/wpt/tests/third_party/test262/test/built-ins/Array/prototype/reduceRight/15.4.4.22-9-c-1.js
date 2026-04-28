// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - callbackfn not called for indexes
    never been assigned values
---*/

var callCnt = 0;

function callbackfn(prevVal, curVal, idx, obj)
{
  callCnt++;
  return curVal;
}

var arr = new Array(10);
arr[0] = arr[1] = undefined; //explicitly assigning a value

assert.sameValue(arr.reduceRight(callbackfn), undefined, 'arr.reduceRight(callbackfn)');
assert.sameValue(callCnt, 1, 'callCnt');
