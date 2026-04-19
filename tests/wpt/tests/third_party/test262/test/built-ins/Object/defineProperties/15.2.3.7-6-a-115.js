// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-115
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is absent, test
    every field in 'desc' is same with corresponding attribute value
    of the length property in 'O' (15.4.5.1 step 3.a.i)
includes: [propertyHelper.js]
---*/

var arr = [];

Object.defineProperties(arr, {
  length: {
    writable: true,
    enumerable: false,
    configurable: false
  }
});

assert.sameValue(arr.length, 0);

arr.length = 2;

verifyProperty(arr, "length", {
  value: 2,
  enumerable: false,
  configurable: false,
});
