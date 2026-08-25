// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element changed by getter on previous
    iterations is observed on an Array
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return val === 12;
  }
  return false;
}

var arr = [];
var helpVerifyVar = 11;

Object.defineProperty(arr, "1", {
  get: function() {
    return helpVerifyVar;
  },
  set: function(args) {
    helpVerifyVar = args;
  },
  configurable: true
});

Object.defineProperty(arr, "0", {
  get: function() {
    arr[1] = 12;
    return 9;
  },
  configurable: true
});

assert(arr.some(callbackfn), 'arr.some(callbackfn) !== true');
