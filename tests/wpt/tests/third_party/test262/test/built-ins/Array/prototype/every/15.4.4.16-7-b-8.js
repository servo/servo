// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - deleting own property causes index
    property not to be visited on an Array-like object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return idx !== 1;
}
var obj = {
  length: 2
};

Object.defineProperty(obj, "1", {
  get: function() {
    return 6.99;
  },
  configurable: true
});

Object.defineProperty(obj, "0", {
  get: function() {
    delete obj[1];
    return 0;
  },
  configurable: true
});

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert(accessed, 'accessed !== true');
