// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - value of 'length' is boundary value
    (2^32 + 1)
---*/

var targetObj = {};
var obj = {
  0: targetObj,
  1: 4294967297,
  length: 4294967297
};

assert.sameValue(Array.prototype.indexOf.call(obj, targetObj), 0, 'Array.prototype.indexOf.call(obj, targetObj)');
assert.sameValue(Array.prototype.indexOf.call(obj, 4294967297), 1, 'Array.prototype.indexOf.call(obj, 4294967297)');
