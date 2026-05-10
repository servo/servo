// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - added properties in step 2 are visible here
---*/

function callbackfn(val, idx, obj) {
  if (idx === 2 && val === "length") {
    return false;
  } else {
    return true;
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

assert.sameValue(Array.prototype.every.call(arr, callbackfn), false, 'Array.prototype.every.call(arr, callbackfn)');
