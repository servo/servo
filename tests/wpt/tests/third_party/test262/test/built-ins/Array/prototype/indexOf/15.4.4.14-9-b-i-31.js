// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - terminates iteration on unhandled
    exception on an Array-like object
---*/

var accessed = false;
var obj = {
  length: 2
};

Object.defineProperty(obj, "0", {
  get: function() {
    throw new TypeError();
  },
  configurable: true
});

Object.defineProperty(obj, "1", {
  get: function() {
    accessed = true;
    return true;
  },
  configurable: true
});
assert.throws(TypeError, function() {
  Array.prototype.indexOf.call(obj, true);
});
assert.sameValue(accessed, false, 'accessed');
