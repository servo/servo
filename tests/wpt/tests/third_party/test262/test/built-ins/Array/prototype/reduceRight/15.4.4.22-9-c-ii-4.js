// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - k values are passed in acending
    numeric order
---*/

var arr = [0, 1, 2, 3, 4, 5];
var lastIdx = arr.length - 1;
var accessed = false;
var result = true;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  if (lastIdx !== idx) {
    result = false;
  } else {
    lastIdx--;
  }
}
arr.reduceRight(callbackfn, 1);

assert(result, 'result !== true');
assert(accessed, 'accessed !== true');
