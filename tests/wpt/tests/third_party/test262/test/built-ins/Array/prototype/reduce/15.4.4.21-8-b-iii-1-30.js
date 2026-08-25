// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element changed by getter on current
    iterations is observed in subsequent iterations on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === 1);
  }
}

var arr = [, , 2];
var preIterVisible = false;

Object.defineProperty(arr, "0", {
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
      return 100;
    }
  },
  configurable: true
});

arr.reduce(callbackfn);

assert(testResult, 'testResult !== true');
