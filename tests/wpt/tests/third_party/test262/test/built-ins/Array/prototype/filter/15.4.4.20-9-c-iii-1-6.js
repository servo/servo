// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - values of 'to' are accessed during each
    iteration when 'selected' is converted to true and not prior to
    starting the loop
---*/

var toIndex = [];
var called = 0;

//By below way, we could verify that 'to' would be setted as 0, 1, ..., length - 1 in order, and each value will be setted one time.
function callbackfn(val, idx, obj) {
  called++;
  //Each position should be visited one time, which means 'to' is accessed one time during iterations.
  if (toIndex[idx] === undefined) {
    //when current position is visited, its previous index should has been visited.
    if (idx !== 0 && toIndex[idx - 1] === undefined) {
      return false;
    }
    toIndex[idx] = 1;
    return true;
  } else {
    return false;
  }
}
var newArr = [11, 12, 13, 14].filter(callbackfn, undefined);

assert.sameValue(newArr.length, 4, 'newArr.length');
assert.sameValue(called, 4, 'called');
