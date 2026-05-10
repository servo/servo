// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - k values are accessed during each
    iteration and not prior to starting the loop on an Array
---*/

var arr = [11, 12, 13, 14];
var kIndex = [];
var result = true;
var called = 0;

//By below way, we could verify that k would be setted as 0, 1, ..., length - 1 in order, and each value will be setted one time.
function callbackfn(preVal, curVal, idx, o) {
  //Each position should be visited one time, which means k is accessed one time during iterations.
  called++;
  if (typeof kIndex[idx] === "undefined") {
    //when current position is visited, its next index should has been visited.
    if (idx !== arr.length - 1 && typeof kIndex[idx + 1] === "undefined") {
      result = false;
    }
    kIndex[idx] = 1;
  } else {
    result = false;
  }
}

arr.reduceRight(callbackfn, 1);

assert(result, 'result !== true');
assert.sameValue(called, 4, 'called');
