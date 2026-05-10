// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is a number (value
    is a positive number)
---*/

var obj = {
  99: true,
  100: 100,
  length: 100
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, true), 99, 'Array.prototype.lastIndexOf.call(obj, true)');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, 100), -1, 'Array.prototype.lastIndexOf.call(obj, 100)');
