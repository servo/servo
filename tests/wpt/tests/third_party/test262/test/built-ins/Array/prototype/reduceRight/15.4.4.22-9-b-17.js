// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - properties added into own object are
    visited on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 0 && curVal === 0) {
    testResult = true;
  }
}

var obj = {
  length: 2
};

Object.defineProperty(obj, "1", {
  get: function() {
    Object.defineProperty(obj, "0", {
      get: function() {
        return 0;
      },
      configurable: true
    });
    return 1;
  },
  configurable: true
});

Array.prototype.reduceRight.call(obj, callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
