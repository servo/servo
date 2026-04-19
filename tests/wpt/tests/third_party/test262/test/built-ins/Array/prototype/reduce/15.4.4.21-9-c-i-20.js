// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element to be retrieved is own accessor
    property without a get function that overrides an inherited
    accessor property on an Array
---*/

var testResult = false;
var initialValue = 0;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === undefined);
  }
}

Object.defineProperty(Array.prototype, "1", {
  get: function() {
    return 11;
  },
  configurable: true
});

var arr = [0, , 2];
Object.defineProperty(arr, "1", {
  set: function() {},
  configurable: true
});

arr.reduce(callbackfn, initialValue);

assert(testResult, 'testResult !== true');
