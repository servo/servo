// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-125
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', test that RangeError exception is thrown when
    [[Value]] field of 'desc' is undefined (15.4.5.1 step 3.c)
---*/

var arrObj = [];
assert.throws(RangeError, function() {
  Object.defineProperty(arrObj, "length", {
    value: undefined
  });
});
