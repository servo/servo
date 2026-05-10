// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is own accessor
    property that overrides an inherited data property on an Array
---*/

var kValue = "abc";

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return val === kValue;
  }
  return false;
}

var arr = [];

Array.prototype[1] = 100;
Object.defineProperty(arr, "1", {
  get: function() {
    return kValue;
  },
  configurable: true
});

assert(arr.some(callbackfn), 'arr.some(callbackfn) !== true');
