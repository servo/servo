// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - the exception is not thrown if exception
    was thrown by step 2
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return (curVal > 10);
}

var obj = {
  0: 11,
  1: 12
};

Object.defineProperty(obj, "length", {
  get: function() {
    throw new Test262Error();
  },
  configurable: true
});

assert.throws(Test262Error, function() {
  Array.prototype.reduce.call(obj, callbackfn);
});
