// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - decreasing length of array causes index
    property not to be visited
---*/

var accessed = false;
var testResult = true;

function callbackfn(val, idx, obj) {
  accessed = true;
  if (idx === 3) {
    testResult = false;
  }
}

var arr = [0, 1, 2, "last"];

Object.defineProperty(arr, "0", {
  get: function() {
    arr.length = 3;
    return 0;
  },
  configurable: true
});

arr.forEach(callbackfn);

assert(testResult, 'testResult !== true');
assert(accessed, 'accessed !== true');
