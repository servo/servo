// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'length' is a string
    containing a decimal number
---*/

var obj = {
  4: 4,
  5: 5,
  length: "5.512345"
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, 4), 4, 'Array.prototype.lastIndexOf.call(obj, 4)');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, 5), -1, 'Array.prototype.lastIndexOf.call(obj, 5)');
