// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-118
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', the [[Value]] field of 'desc' is absent, test
    every field in 'desc' is absent (15.4.5.1 step 3.a.i)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, "length", {});

assert.sameValue(arrObj.length, 0);
arrObj.length = 2;

verifyProperty(arrObj, "length", {
  value: 2,
  enumerable: false,
  configurable: false,
});
