// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is boundary value
    (2^32)
---*/

var targetObj = {};
var obj = {
  0: targetObj,
  4294967294: targetObj,
  4294967295: targetObj,
  length: 4294967296
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, targetObj), 4294967295, 'verify length is 4294967296 finally');
