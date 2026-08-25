// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf applied to the Arguments object
---*/

var obj = (function fun() {
  return arguments;
}(1, 2, 3));

assert.sameValue(Array.prototype.lastIndexOf.call(obj, 2), 1, 'Array.prototype.lastIndexOf.call(obj, 2)');
