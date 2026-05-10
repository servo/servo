// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is own
    accessor property without a get function that overrides an
    inherited accessor property on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (typeof prevVal === "undefined");
  }
}

Array.prototype[2] = 2;
var arr = [0, 1];
Object.defineProperty(arr, "2", {
  set: function() {},
  configurable: true
});

arr.reduceRight(callbackfn);

assert(testResult, 'testResult !== true');
