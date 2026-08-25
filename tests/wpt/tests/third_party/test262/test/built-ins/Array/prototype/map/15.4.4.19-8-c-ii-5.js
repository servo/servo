// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - k values are accessed during each iteration
    and not prior to starting the loop.
---*/

var kIndex = [];

//By below way, we could verify that k would be setted as 0, 1, ..., length - 1 in order, and each value will be setted one time.
function callbackfn(val, idx, obj) {
  //Each position should be visited one time, which means k is accessed one time during iterations.
  if (typeof kIndex[idx] === "undefined") {
    //when current position is visited, its previous index should has been visited.
    if (idx !== 0 && typeof kIndex[idx - 1] === "undefined") {
      return true;
    }
    kIndex[idx] = 1;
    return false;
  } else {
    return true;
  }
}

var testResult = [11, 12, 13, 14].map(callbackfn);

assert.sameValue(testResult.length, 4, 'testResult.length');
assert.sameValue(testResult[0], false, 'testResult[0]');
assert.sameValue(testResult[1], false, 'testResult[1]');
assert.sameValue(testResult[2], false, 'testResult[2]');
assert.sameValue(testResult[3], false, 'testResult[3]');
