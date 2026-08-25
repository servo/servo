// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is inherited
    accessor property on an Array
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === 1);
  }
}

Object.defineProperty(Array.prototype, "1", {
  get: function() {
    return 1;
  },
  configurable: true
});

var arr = [0, , 2];

arr.reduceRight(callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
