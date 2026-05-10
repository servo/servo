// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - decreasing length of array causes index
    property not to be visited
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return idx !== 3;
}
var arr = [0, 1, 2, "last"];

Object.defineProperty(arr, "0", {
  get: function() {
    arr.length = 3;
    return 0;
  },
  configurable: true
});

assert(arr.every(callbackfn), 'arr.every(callbackfn) !== true');
assert(accessed, 'accessed !== true');
