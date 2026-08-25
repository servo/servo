// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element changed by getter on previous
    iterations is observed on an Array-like object
---*/

var testResult = false;
var initialValue = 0;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === 1);
  }
}

var obj = {
  2: 2,
  length: 3
};
var preIterVisible = false;

Object.defineProperty(obj, "0", {
  get: function() {
    preIterVisible = true;
    return 0;
  },
  configurable: true
});

Object.defineProperty(obj, "1", {
  get: function() {
    if (preIterVisible) {
      return 1;
    } else {
      return "11";
    }
  },
  configurable: true
});

Array.prototype.reduce.call(obj, callbackfn, initialValue);

assert(testResult, 'testResult !== true');
