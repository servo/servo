// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - exception in getter terminates iteration
    on an Array
---*/

var accessed = false;
var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx >= 1) {
    accessed = true;
    testResult = (prevVal === 0);
  }
}

var arr = [, 1, 2];

Object.defineProperty(arr, "0", {
  get: function() {
    throw new RangeError("unhandle exception happened in getter");
  },
  configurable: true
});
assert.throws(RangeError, function() {
  arr.reduce(callbackfn);
});
assert.sameValue(accessed, false, 'accessed');
assert.sameValue(testResult, false, 'testResult');
