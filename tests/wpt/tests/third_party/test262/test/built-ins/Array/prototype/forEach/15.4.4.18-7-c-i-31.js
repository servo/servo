// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - unnhandled exceptions happened in getter
    terminate iteration on an Array-like object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  if (idx > 1) {
    accessed = true;
  }
}

var arr = [];
arr[5] = 10;
arr[10] = 100;

Object.defineProperty(arr, "1", {
  get: function() {
    throw new RangeError("unhandle exception happened in getter");
  },
  configurable: true
});

Object.defineProperty(arr, "2", {
  get: function() {
    accessed = true;
    return 100;
  },
  configurable: true
});
assert.throws(RangeError, function() {
  arr.forEach(callbackfn);
});
assert.sameValue(accessed, false, 'accessed');
