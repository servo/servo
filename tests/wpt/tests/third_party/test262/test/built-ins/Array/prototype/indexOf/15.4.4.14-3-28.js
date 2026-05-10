// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - value of 'length' is boundary value
    (2^32)
---*/

var targetObj = {};
var obj = {
  0: targetObj,
  4294967294: targetObj,
  4294967295: targetObj,
  length: 4294967296
};

assert.sameValue(Array.prototype.indexOf.call(obj, targetObj), 0, 'Array.prototype.indexOf.call(obj, targetObj)');
