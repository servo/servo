// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - element changed by getter on previous
    iterations is observed on an Array
---*/

var preIterVisible = false;
var arr = [];

function callbackfn(val, idx, obj) {
  return idx === 1 && val === 9;
}

Object.defineProperty(arr, "0", {
  get: function() {
    preIterVisible = true;
    return 11;
  },
  configurable: true
});

Object.defineProperty(arr, "1", {
  get: function() {
    if (preIterVisible) {
      return 9;
    } else {
      return 11;
    }
  },
  configurable: true
});
var newArr = arr.filter(callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 9, 'newArr[0]');
