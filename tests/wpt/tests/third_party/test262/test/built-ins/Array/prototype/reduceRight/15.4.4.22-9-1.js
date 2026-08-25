// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight doesn't consider new elements which
    index is larger than array original length added to array after it
    is called, consider new elements which index is smaller than array
    length
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  arr[5] = 6;
  arr[2] = 3;
  return prevVal + curVal;
}

var arr = ['1', 2, , 4, '5'];

assert.sameValue(arr.reduceRight(callbackfn), "54321", 'arr.reduceRight(callbackfn)');
