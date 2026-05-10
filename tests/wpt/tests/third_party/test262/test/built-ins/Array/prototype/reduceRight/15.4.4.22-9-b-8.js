// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - deleting own property in step 8
    causes deleted index property not to be visited on an Array-like
    object
---*/

var accessed = false;
var testResult = true;

function callbackfn(preVal, val, idx, obj) {
  accessed = true;
  if (idx === 1) {
    testResult = false;
  }
}

var obj = {
  0: 10,
  length: 10
};

Object.defineProperty(obj, "1", {
  get: function() {
    return 6.99;
  },
  configurable: true
});

Object.defineProperty(obj, "5", {
  get: function() {
    delete obj[1];
    return 0;
  },
  configurable: true
});

Array.prototype.reduceRight.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
assert(accessed, 'accessed !== true');
