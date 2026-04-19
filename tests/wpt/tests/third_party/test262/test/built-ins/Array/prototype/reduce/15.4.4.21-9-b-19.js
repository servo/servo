// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - properties added to prototype are visited
    on an Array-like object
---*/

var testResult = false;

function callbackfn(accum, val, idx, obj) {
  if (idx === 1 && val === 6.99) {
    testResult = true;
  }
}

var obj = {
  length: 6
};

Object.defineProperty(obj, "0", {
  get: function() {
    Object.defineProperty(Object.prototype, "1", {
      get: function() {
        return 6.99;
      },
      configurable: true
    });
    return 0;
  },
  configurable: true
});

Array.prototype.reduce.call(obj, callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
