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
    testResult = (prevVal === 2);
  }
}

Object.defineProperty(Array.prototype, "2", {
  get: function() {
    return 2;
  },
  configurable: true
});

var arr = [0, 1, , ];

arr.reduceRight(callbackfn);

assert(testResult, 'testResult !== true');
