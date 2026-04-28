// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - decreasing length of array causes index
    property not to be visited
---*/

function callbackfn(val, idx, obj) {
  return idx === 3 && typeof val === "undefined";
}

var arr = [0, 1, 2, "last"];

Object.defineProperty(arr, "0", {
  get: function() {
    arr.length = 3;
    return 0;
  },
  configurable: true
});

var testResult = arr.map(callbackfn);

assert.sameValue(typeof testResult[3], "undefined", 'typeof testResult[3]');
