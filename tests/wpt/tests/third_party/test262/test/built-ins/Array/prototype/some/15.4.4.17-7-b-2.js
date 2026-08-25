// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - added properties in step 2 are visible here
---*/

function callbackfn(val, idx, obj) {
  if (idx === 2 && val === "length") {
    return true;
  } else {
    return false;
  }
}

var arr = {};

Object.defineProperty(arr, "length", {
  get: function() {
    arr[2] = "length";
    return 3;
  },
  configurable: true
});

assert(Array.prototype.some.call(arr, callbackfn), 'Array.prototype.some.call(arr, callbackfn) !== true');
