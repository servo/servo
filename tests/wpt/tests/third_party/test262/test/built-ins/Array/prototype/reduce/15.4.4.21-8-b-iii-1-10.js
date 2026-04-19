// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - when element to be retrieved is own
    accessor property on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === 0);
  }
}

var arr = [, 1, 2];

Object.defineProperty(arr, "0", {
  get: function() {
    return 0;
  },
  configurable: true
});

arr.reduce(callbackfn);

assert(testResult, 'testResult !== true');
