// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-120
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is absent, test
    updating the [[Writable]] attribute of the length property from
    true to false (15.4.5.1 step 3.a.i)
includes: [propertyHelper.js]
---*/

var arr = [];

Object.defineProperties(arr, {
  length: {
    writable: false
  }
});

verifyProperty(arr, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: false,
});
