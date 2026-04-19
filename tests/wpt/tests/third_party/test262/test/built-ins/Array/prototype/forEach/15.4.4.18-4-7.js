// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach throws TypeError if callbackfn is Object
    without Call internal method
---*/

var arr = new Array(10);
assert.throws(TypeError, function() {
  arr.forEach(new Object());
});
