// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - element changed by getter on previous
    iterations is observed on an Array
---*/

var preIterVisible = false;
var arr = [];

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 11;
  } else if (idx === 1) {
    return val === 9;
  } else {
    return false;
  }
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

var testResult = arr.map(callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
assert.sameValue(testResult[1], true, 'testResult[1]');
