// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element to be retrieved is own data
    property that overrides an inherited accessor property on an Array
---*/

var testResult = false;
var initialValue = 0;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === 1);
  }
}

Object.defineProperty(Array.prototype, "1", {
  get: function() {
    return "9";
  },
  configurable: true
});
[0, 1, 2].reduce(callbackfn, initialValue);

assert(testResult, 'testResult !== true');
