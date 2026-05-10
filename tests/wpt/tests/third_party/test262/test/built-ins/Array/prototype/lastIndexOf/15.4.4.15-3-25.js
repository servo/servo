// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is a negative
    non-integer
---*/

var obj = {
  1: true,
  2: false,
  length: -4294967294.5
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, true), -1, 'Array.prototype.lastIndexOf.call(obj, true)');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, false), -1, 'Array.prototype.lastIndexOf.call(obj, false)');
