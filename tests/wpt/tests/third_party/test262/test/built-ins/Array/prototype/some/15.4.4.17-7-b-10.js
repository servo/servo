// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - deleting property of prototype causes
    prototype index property not to be visited on an Array-like Object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return idx === 1;
}
var arr = {
  2: 2,
  length: 20
};

Object.defineProperty(arr, "0", {
  get: function() {
    delete Object.prototype[1];
    return 0;
  },
  configurable: true
});

Object.prototype[1] = 1;

assert.sameValue(Array.prototype.some.call(arr, callbackfn), false, 'Array.prototype.some.call(arr, callbackfn)');
assert(accessed, 'accessed !== true');
