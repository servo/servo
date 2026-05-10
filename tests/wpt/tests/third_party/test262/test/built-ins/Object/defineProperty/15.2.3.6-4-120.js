// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-120
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' is absent, test
    TypeError is thrown when updating the [[Configurable]] attribute
    of the length property from false to true (15.4.5.1 step 3.a.i)
---*/

var arrObj = [];
assert.throws(TypeError, function() {
  Object.defineProperty(arrObj, "length", {
    configurable: true
  });
});
