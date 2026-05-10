// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - properties added to prototype in step 8
    are visited on an Array
---*/

var testResult = false;

function callbackfn(accum, val, idx, obj) {
  if (idx === 1 && val === 6.99) {
    testResult = true;
  }
}

var arr = [0, , 2];

Object.defineProperty(arr, "0", {
  get: function() {
    Object.defineProperty(Array.prototype, "1", {
      get: function() {
        return 6.99;
      },
      configurable: true
    });
    return 0;
  },
  configurable: true
});

arr.reduce(callbackfn);

assert(testResult, 'testResult !== true');
