// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - properties added into own object in
    step 8 can be visited on an Array
---*/

var testResult = false;

function callbackfn(preVal, curVal, idx, obj) {
  if (idx === 1 && curVal === 1) {
    testResult = true;
  }
}

var arr = [0, , 2];

Object.defineProperty(arr, "2", {
  get: function() {
    Object.defineProperty(arr, "1", {
      get: function() {
        return 1;
      },
      configurable: true
    });
    return 0;
  },
  configurable: true
});

arr.reduceRight(callbackfn);

assert(testResult, 'testResult !== true');
