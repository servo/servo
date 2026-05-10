// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - decreasing length of array with prototype
    property in step 8 causes prototype index property to be visited
---*/

var testResult = false;

function callbackfn(accum, val, idx, obj) {
  if (idx === 2 && val === "prototype") {
    testResult = true;
  }
}
var arr = [0, 1, 2, 3];

Object.defineProperty(Array.prototype, "2", {
  get: function() {
    return "prototype";
  },
  configurable: true
});

Object.defineProperty(arr, "0", {
  get: function() {
    arr.length = 2;
    return 1;
  },
  configurable: true
});

arr.reduce(callbackfn);

assert(testResult, 'testResult !== true');
