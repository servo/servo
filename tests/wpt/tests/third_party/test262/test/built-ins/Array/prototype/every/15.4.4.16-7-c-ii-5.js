// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - k values are accessed during each
    iteration and not prior to starting the loop on an Array
---*/

var called = 0;
var kIndex = [];

//By below way, we could verify that k would be setted as 0, 1, ..., length - 1 in order, and each value will be setted one time.
function callbackfn(val, idx, obj) {
  called++;
  //Each position should be visited one time, which means k is accessed one time during iterations.
  if (typeof kIndex[idx] === "undefined") {
    //when current position is visited, its previous index should has been visited.
    if (idx !== 0 && typeof kIndex[idx - 1] === "undefined") {
      return false;
    }
    kIndex[idx] = 1;
    return true;
  } else {
    return false;
  }
}

assert([11, 12, 13, 14].every(callbackfn, undefined), '[11, 12, 13, 14].every(callbackfn, undefined) !== true');
assert.sameValue(called, 4, 'called');
