// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is own
    accessor property that overrides an inherited accessor property on
    an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === "20");
  }
}

Object.defineProperty(Array.prototype, "2", {
  get: function() {
    return 2;
  },
  configurable: true
});

var arr = [0, 1, , ];

Object.defineProperty(arr, "2", {
  get: function() {
    return "20";
  },
  configurable: true
});

arr.reduceRight(callbackfn);

assert(testResult, 'testResult !== true');
