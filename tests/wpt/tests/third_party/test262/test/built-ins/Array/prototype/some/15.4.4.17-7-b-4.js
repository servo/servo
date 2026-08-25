// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - properties added into own object after
    current position are visited on an Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1 && val === 1) {
    return true;
  } else {
    return false;
  }
}

var arr = {
  length: 2
};

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

assert(Array.prototype.some.call(arr, callbackfn), 'Array.prototype.some.call(arr, callbackfn) !== true');
