// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - decreasing length of array does not delete
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

assert.sameValue(arr.every(callbackfn), false, 'arr.every(callbackfn)');
