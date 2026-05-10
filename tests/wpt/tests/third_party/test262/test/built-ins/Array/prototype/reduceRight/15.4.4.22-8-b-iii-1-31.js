// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element changed by getter on current
    iteration is observed subsequetly on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === 2 && curVal === 1);
  }
}

var obj = {
  0: 0,
  length: 3
};
var preIterVisible = false;

Object.defineProperty(obj, "1", {
  get: function() {
    if (preIterVisible) {
      return 1;
    } else {
      return "20";
    }
  },
  configurable: true
});

Object.defineProperty(obj, "2", {
  get: function() {
    preIterVisible = true;
    return 2;
  },
  configurable: true
});

Array.prototype.reduceRight.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
