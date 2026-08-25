// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - deleting own property in step 8 causes
    deleted index property not to be visited on an Array
---*/

var accessed = false;
var testResult = true;

function callbackfn(accum, val, idx, obj) {
  accessed = true;
  if (idx === 1) {
    testResult = false;
  }
}

var arr = [1, 2, 4];

Object.defineProperty(arr, "0", {
  get: function() {
    delete arr[1];
    return 0;
  },
  configurable: true
});

arr.reduce(callbackfn);

assert(testResult, 'testResult !== true');
assert(accessed, 'accessed !== true');
