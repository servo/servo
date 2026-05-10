// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-198
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' property doesn't exist in 'O', test
    TypeError is thrown when 'O' is not extensible (15.4.5.1 step 4.c)
---*/

var arrObj = [];
Object.preventExtensions(arrObj);
assert.throws(TypeError, function() {
  var desc = {
    value: 1
  };
  Object.defineProperty(arrObj, "0", desc);
});
assert.sameValue(arrObj.hasOwnProperty("0"), false, 'arrObj.hasOwnProperty("0")');
