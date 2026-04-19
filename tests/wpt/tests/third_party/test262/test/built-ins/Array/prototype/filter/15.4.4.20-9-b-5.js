// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-5
description: >
    Array.prototype.filter - properties added into own object after
    current position are visited on an Array
---*/

function callbackfn(val, idx, obj) {
  return true;
}

var arr = [0, , 2];

Object.defineProperty(arr, "0", {
  get: function() {
    Object.defineProperty(arr, "1", {
      get: function() {
        return 6.99;
      },
      configurable: true
    });
    return 0;
  },
  configurable: true
});

var newArr = arr.filter(callbackfn);

assert.sameValue(newArr.length, 3, 'newArr.length');
assert.sameValue(newArr[1], 6.99, 'newArr[1]');
