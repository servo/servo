// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element changed by getter on
    previous iterations is observed on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === 1);
  }
}

var arr = [, , ];
var preIterVisible = false;

Object.defineProperty(arr, "2", {
  get: function() {
    preIterVisible = true;
    return 0;
  },
  configurable: true
});

Object.defineProperty(arr, "1", {
  get: function() {
    if (preIterVisible) {
      return 1;
    } else {
      return "11";
    }
  },
  configurable: true
});

arr.reduceRight(callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
