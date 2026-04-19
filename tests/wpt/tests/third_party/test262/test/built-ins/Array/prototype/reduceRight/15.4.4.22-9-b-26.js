// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - deleting own property with prototype
    property causes prototype index property to be visited on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1 && curVal === 1) {
    testResult = true;
  }
}
var arr = [0, 111];

Object.defineProperty(arr, "2", {
  get: function() {
    delete arr[1];
    return 0;
  },
  configurable: true
});

Array.prototype[1] = 1;
arr.reduceRight(callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
