// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - decreasing length of array does not delete
    non-configurable properties
flags: [noStrict]
---*/

function callbackfn(val, idx, obj) {
  if (idx === 2 && val === "unconfigurable") {
    return false;
  } else {
    return true;
  }
}

var arr = [0, 1, 2];

Object.defineProperty(arr, "2", {
  get: function() {
    return "unconfigurable";
  },
  configurable: false
});

Object.defineProperty(arr, "1", {
  get: function() {
    arr.length = 2;
    return 1;
  },
  configurable: true
});

var testResult = arr.map(callbackfn);

assert.sameValue(testResult.length, 3, 'testResult.length');
assert.sameValue(testResult[2], false, 'testResult[2]');
