// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - if exception occurs, it occurs after any
    side-effects that might be produced by step 3
---*/

var obj = {};

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
  Array.prototype.reduce.call(obj, function() {});
});
assert(accessed, 'accessed !== true');
