// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - callbackfn called with correct
    parameters (initialvalue passed)
---*/

var bParCorrect = false;
var arr = [0, 1, true, null, new Object(), "five"];
var initialValue = 5.5;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === obj.length - 1 && obj[idx] === curVal && prevVal === initialValue)
    return curVal;
  else if (idx + 1 < obj.length && obj[idx] === curVal && obj[idx + 1] === prevVal)
    return curVal;
  else
    return false;
}

assert.sameValue(arr.reduceRight(callbackfn, initialValue), 0, 'arr.reduceRight(callbackfn, initialValue)');
