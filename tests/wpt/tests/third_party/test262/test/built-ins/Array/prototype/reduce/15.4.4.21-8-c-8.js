// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - the exception is not thrown if exception
    was thrown by step 3
---*/

var obj = {};

Object.defineProperty(obj, "length", {
  get: function() {
    return {
      toString: function() {
        throw new Test262Error();
      }
    };
  },
  configurable: true
});

assert.throws(Test262Error, function() {
  Array.prototype.reduce.call(obj, function() {});
});
