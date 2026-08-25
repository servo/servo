// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-129
description: >
    Object.defineProperties - 'O' is an Array, 'name' is the length
    property of 'O', test the [[Value]] field of 'desc' is negative
    number (15.4.5.1 step 3.c)
---*/

var arr = [];
assert.throws(RangeError, function() {
  Object.defineProperties(arr, {
    length: {
      value: -9
    }
  });
});
assert.sameValue(arr.length, 0, 'arr.length');
