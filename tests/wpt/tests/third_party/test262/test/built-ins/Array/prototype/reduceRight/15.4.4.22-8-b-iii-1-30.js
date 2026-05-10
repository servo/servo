// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element changed by getter on current
    iteration is observed in subsequent iterations on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === 1 && prevVal === 2);
  }
}

var arr = [0];
var preIterVisible = false;

Object.defineProperty(arr, "1", {
  get: function() {
    if (preIterVisible) {
      return 1;
    } else {
      return "20";
    }
  },
  configurable: true
});

Object.defineProperty(arr, "2", {
  get: function() {
    preIterVisible = true;
    return 2;
  },
  configurable: true
});

arr.reduceRight(callbackfn);

assert(testResult, 'testResult !== true');
