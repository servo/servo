// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - decreasing length of array with
    prototype property causes prototype index property to be visited
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 2 && curVal === "prototype") {
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

Object.defineProperty(arr, "3", {
  get: function() {
    arr.length = 2;
    return 1;
  },
  configurable: true
});

arr.reduceRight(callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
