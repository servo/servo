// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - properties added into own object after
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

var testResult = arr.map(callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
assert.sameValue(testResult[1], false, 'testResult[1]');
