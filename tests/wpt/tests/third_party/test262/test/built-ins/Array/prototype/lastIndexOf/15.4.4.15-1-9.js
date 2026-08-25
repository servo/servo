// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf applied to Function object
---*/

var obj = function(a, b) {
  return a + b;
};
obj[1] = true;

assert.sameValue(Array.prototype.lastIndexOf.call(obj, true), 1, 'Array.prototype.lastIndexOf.call(obj, true)');
