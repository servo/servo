// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own accessor
    property on an Array
---*/

function callbackfn(val, idx, obj) {
  if (idx === 2) {
    return val !== 12;
  } else {
    return true;
  }
}

var arr = [];

Object.defineProperty(arr, "2", {
  get: function() {
    return 12;
  },
  configurable: true
});

assert.sameValue(arr.every(callbackfn), false, 'arr.every(callbackfn)');
