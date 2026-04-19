// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - 'length' is own data property on an
    Array-like object
---*/

var objOne = {
  1: true,
  length: 2
};
var objTwo = {
  2: true,
  length: 2
};

assert.sameValue(Array.prototype.indexOf.call(objOne, true), 1, 'Array.prototype.indexOf.call(objOne, true)');
assert.sameValue(Array.prototype.indexOf.call(objTwo, true), -1, 'Array.prototype.indexOf.call(objTwo, true)');
