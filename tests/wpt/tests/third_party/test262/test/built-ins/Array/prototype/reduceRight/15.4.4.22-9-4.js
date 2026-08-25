// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight doesn't consider unvisited deleted
    elements when Array.length is decreased
---*/

function callbackfn(prevVal, curVal, idx, obj)
{
  arr.length = 2;
  return prevVal + curVal;
}

var arr = [1, 2, 3, 4, 5];

assert.sameValue(arr.reduceRight(callbackfn), 12, 'arr.reduceRight(callbackfn)');
