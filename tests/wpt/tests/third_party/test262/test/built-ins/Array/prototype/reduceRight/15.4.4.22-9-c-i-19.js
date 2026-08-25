// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is own
    accessor property without a get function that overrides an
    inherited accessor property on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (typeof curVal === "undefined");
  }
}

Object.prototype[1] = 1;

var obj = {
  0: 0,
  2: 2,
  length: 3
};
Object.defineProperty(obj, "1", {
  set: function() {},
  configurable: true
});

Array.prototype.reduceRight.call(obj, callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
