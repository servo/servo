// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - Side effects produced by step 3 are visible
    when an exception occurs
---*/

var obj = {
  0: 11,
  1: 12
};

var accessed = false;

Object.defineProperty(obj, "length", {
  get: function() {
    return {
      toString: function() {
        accessed = true;
        return "2";
      }
    };
  },
  configurable: true
});
assert.throws(TypeError, function() {
  Array.prototype.map.call(obj, null);
});
assert(accessed, 'accessed !== true');
