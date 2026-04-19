// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - unnhandled exceptions happened in getter
    terminate iteration on an Array-like object
---*/

var accessed = false;
var testResult = false;
var initialValue = 0;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx >= 1) {
    accessed = true;
    testResult = (curVal >= 1);
  }
}

var obj = {
  0: 0,
  2: 2,
  length: 3
};
Object.defineProperty(obj, "1", {
  get: function() {
    throw new RangeError("unhandle exception happened in getter");
  },
  configurable: true
});
assert.throws(RangeError, function() {
  Array.prototype.reduce.call(obj, callbackfn, initialValue);
});
assert.sameValue(accessed, false, 'accessed');
assert.sameValue(testResult, false, 'testResult');
