// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is own
    accessor property on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === 2);
  }
}

var obj = {
  0: 0,
  1: 1,
  length: 3
};
Object.defineProperty(obj, "2", {
  get: function() {
    return 2;
  },
  configurable: true
});

Array.prototype.reduceRight.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
