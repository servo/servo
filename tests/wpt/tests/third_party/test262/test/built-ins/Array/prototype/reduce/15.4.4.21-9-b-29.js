// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - decreasing length of array does not
    delete non-configurable properties
flags: [noStrict]
---*/

var testResult = false;

function callbackfn(accum, val, idx, obj) {
  if (idx === 2 && val === "unconfigurable") {
    testResult = true;
  }
}

var arr = [0, 1, 2, 3];

Object.defineProperty(arr, "2", {
  get: function() {
    return "unconfigurable";
  },
  configurable: false
});

Object.defineProperty(arr, "0", {
  get: function() {
    arr.length = 2;
    return 1;
  },
  configurable: true
});

arr.reduce(callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
