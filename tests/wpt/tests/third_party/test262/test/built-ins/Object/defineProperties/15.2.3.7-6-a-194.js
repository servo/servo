// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-194
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' property doesn't exist in 'O', test TypeError
    is thrown when 'O' is not extensible  (15.4.5.1 step 4.c)
---*/

var arr = [];
Object.preventExtensions(arr);
assert.throws(TypeError, function() {
  Object.defineProperties(arr, {
    "0": {
      value: 1
    }
  });
});
assert.sameValue(arr.hasOwnProperty("0"), false, 'arr.hasOwnProperty("0")');
