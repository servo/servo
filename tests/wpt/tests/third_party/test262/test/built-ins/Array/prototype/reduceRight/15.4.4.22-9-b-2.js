// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - added properties in step 2 are
    visible here
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 2 && curVal === "2") {
    testResult = true;
  }
}

var obj = {};

Object.defineProperty(obj, "length", {
  get: function() {
    obj[2] = "2";
    return 3;
  },
  configurable: true
});

Array.prototype.reduceRight.call(obj, callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
