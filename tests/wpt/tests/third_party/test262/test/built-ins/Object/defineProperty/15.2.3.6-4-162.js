// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-162
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' is greater than
    value of the length property, test TypeError is thrown when the
    length property is not writable (15.4.5.1 step 3.f.i)
---*/

var arrObj = [];

Object.defineProperty(arrObj, "length", {
  writable: false
});
assert.throws(TypeError, function() {
  Object.defineProperty(arrObj, "length", {
    value: 12
  });
});
