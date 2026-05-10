// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element to be retrieved is inherited
    accessor property without a get function on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === undefined);
  }
}

Object.defineProperty(Array.prototype, "0", {
  set: function() {},
  configurable: true
});

var arr = [, 1, 2];

arr.reduce(callbackfn);

assert(testResult, 'testResult !== true');
