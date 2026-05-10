// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - decreasing length of array in step 8
    does not delete non-configurable properties
flags: [noStrict]
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 2 && curVal === "unconfigurable") {
    testResult = true;
  }
}

var arr = [0, 1, 2, 3];

Object.defineProperty(arr, "2", {
  get: function() {
    return "unconfigurable";
  },
  configurable: false
});

Object.defineProperty(arr, "3", {
  get: function() {
    arr.length = 2;
    return 1;
  },
  configurable: true
});

arr.reduceRight(callbackfn);

assert(testResult, 'testResult !== true');
