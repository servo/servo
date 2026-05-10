// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is a string
    containing negative number
---*/

var obj = {
  1: null,
  2: undefined,
  length: "-4294967294"
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, null), -1, 'Array.prototype.lastIndexOf.call(obj, null)');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, undefined), -1, 'Array.prototype.lastIndexOf.call(obj, undefined)');
