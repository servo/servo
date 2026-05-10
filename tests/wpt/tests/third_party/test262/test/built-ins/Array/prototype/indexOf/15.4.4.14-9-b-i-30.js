// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - terminates iteration on unhandled
    exception on an Array
---*/

var accessed = false;
var arr = [];

Object.defineProperty(arr, "0", {
  get: function() {
    throw new TypeError();
  },
  configurable: true
});

Object.defineProperty(arr, "1", {
  get: function() {
    accessed = true;
    return true;
  },
  configurable: true
});
assert.throws(TypeError, function() {
  arr.indexOf(true);
});
assert.sameValue(accessed, false, 'accessed');
