// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - decreasing length of array does not delete
    non-configurable properties
flags: [noStrict]
---*/

function callbackfn(val, idx, obj) {
  if (idx === 2 && val === "unconfigurable") {
    return true;
  } else {
    return false;
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

assert(arr.some(callbackfn), 'arr.some(callbackfn) !== true');
