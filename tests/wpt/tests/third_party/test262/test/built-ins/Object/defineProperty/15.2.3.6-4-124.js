// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-124
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' is absent, test
    updating the [[Writable]] attribute of the length property from
    true to false (15.4.5.1 step 3.a.i)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, "length", {
  writable: false
});

verifyProperty(arrObj, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: false,
});
