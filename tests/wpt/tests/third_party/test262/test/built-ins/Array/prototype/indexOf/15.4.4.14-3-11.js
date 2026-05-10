// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - 'length' is a string containing a
    positive number
---*/

var obj = {
  1: 1,
  2: 2,
  length: "2"
};

assert.sameValue(Array.prototype.indexOf.call(obj, 1), 1, 'Array.prototype.indexOf.call(obj, 1)');
assert.sameValue(Array.prototype.indexOf.call(obj, 2), -1, 'Array.prototype.indexOf.call(obj, 2)');
