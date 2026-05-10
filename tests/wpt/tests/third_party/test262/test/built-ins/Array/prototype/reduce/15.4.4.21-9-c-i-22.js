// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element to be retrieved is inherited
    accessor property without a get function on an Array
---*/

var testResult = false;
var initialValue = 0;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === undefined);
  }
}

Object.defineProperty(Array.prototype, "1", {
  set: function() {},
  configurable: true
});

var arr = [0, , 2];

arr.reduce(callbackfn, initialValue);

assert(testResult, 'testResult !== true');
