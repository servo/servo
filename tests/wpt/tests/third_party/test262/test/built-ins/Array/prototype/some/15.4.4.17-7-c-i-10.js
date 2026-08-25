// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is own accessor
    property on an Array
---*/

var kValue = "abc";

function callbackfn(val, idx, obj) {
  if (idx === 10) {
    return val === kValue;
  }
  return false;
}

var arr = [];

Object.defineProperty(arr, "10", {
  get: function() {
    return kValue;
  },
  configurable: true
});

assert(arr.some(callbackfn), 'arr.some(callbackfn) !== true');
