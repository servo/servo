// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - properties can be added to prototype after
    current position are visited on an Array
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1 && val === 6.99) {
    return false;
  } else {
    return true;
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

var testResult = arr.map(callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
assert.sameValue(testResult[1], false, 'testResult[1]');
