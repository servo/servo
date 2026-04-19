// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is own accessor
    property that overrides an inherited accessor property on an Array
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    testResult = (val === 11);
  }
}

var arr = [];

Object.defineProperty(Array.prototype, "0", {
  get: function() {
    return 5;
  },
  configurable: true
});

Object.defineProperty(arr, "0", {
  get: function() {
    return 11;
  },
  configurable: true
});

arr.forEach(callbackfn);

assert(testResult, 'testResult !== true');
