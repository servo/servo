// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - deleting own property with prototype
    property in step 8 causes prototype index property to be visited
    on an Array-like object
---*/

var testResult = false;

function callbackfn(accum, val, idx, obj) {
  if (idx === 1 && val === 1) {
    testResult = true;
  }
}

var obj = {
  0: 0,
  1: 111,
  4: 10,
  length: 10
};

Object.defineProperty(obj, "0", {
  get: function() {
    delete obj[1];
    return 0;
  },
  configurable: true
});

Object.prototype[1] = 1;
Array.prototype.reduce.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
