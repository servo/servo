// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - properties added into own object in step
    8 are visited on an Array-like object
---*/

var testResult = false;

function callbackfn(accum, val, idx, obj) {
  if (idx === 3 && val === 3) {
    testResult = true;
  }
}

var obj = {
  length: 5
};

Object.defineProperty(obj, "1", {
  get: function() {
    Object.defineProperty(obj, "3", {
      get: function() {
        return 3;
      },
      configurable: true
    });
    return 1;
  },
  configurable: true
});

Array.prototype.reduce.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
