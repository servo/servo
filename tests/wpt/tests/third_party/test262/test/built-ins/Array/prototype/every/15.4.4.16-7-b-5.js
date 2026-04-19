// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - properties added into own object after
    current position are visited on an Array
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1 && val === 1) {
    return false;
  } else {
    return true;
  }
}

var arr = [0, , 2];

Object.defineProperty(arr, "0", {
  get: function() {
    Object.defineProperty(arr, "1", {
      get: function() {
        return 1;
      },
      configurable: true
    });
    return 0;
  },
  configurable: true
});

assert.sameValue(arr.every(callbackfn), false, 'arr.every(callbackfn)');
