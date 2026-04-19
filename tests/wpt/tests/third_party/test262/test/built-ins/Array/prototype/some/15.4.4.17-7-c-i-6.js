// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is own data
    property that overrides an inherited accessor property on an Array
---*/

var kValue = 1000;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === kValue;
  }
  return false;
}

Object.defineProperty(Array.prototype, "0", {
  get: function() {
    return 9;
  },
  configurable: true
});

assert([kValue].some(callbackfn), '[kValue].some(callbackfn) !== true');
