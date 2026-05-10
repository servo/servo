// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf applied to String object which
    implements its own property get method
---*/

var obj = function(a, b) {
  return a + b;
};
obj[1] = "b";
obj[2] = "c";

assert.sameValue(Array.prototype.lastIndexOf.call(obj, obj[1]), 1, 'Array.prototype.lastIndexOf.call(obj, obj[1])');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, obj[2]), -1, 'Array.prototype.lastIndexOf.call(obj, obj[2])');
