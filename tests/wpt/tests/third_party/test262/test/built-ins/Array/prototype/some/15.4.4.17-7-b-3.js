// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - deleted properties in step 2 are visible
    here
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return idx === 2;
}
var arr = {
  2: 6.99,
  8: 19
};

Object.defineProperty(arr, "length", {
  get: function() {
    delete arr[2];
    return 10;
  },
  configurable: true
});

assert.sameValue(Array.prototype.some.call(arr, callbackfn), false, 'Array.prototype.some.call(arr, callbackfn)');
assert(accessed, 'accessed !== true');
