// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is own
    accessor property that overrides an inherited data property on an
    Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === "1");
  }
}

Array.prototype[1] = 11;
var arr = [0, , 2];

Object.defineProperty(arr, "1", {
  get: function() {
    return "1";
  },
  configurable: true
});

arr.reduceRight(callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
