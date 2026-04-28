// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-220
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    property, the [[Value]] field of 'desc' and the [[Value]]
    attribute value of 'name' are two numbers with same vaule
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, "0", {
  value: 101
});

Object.defineProperty(arrObj, "0", {
  value: 101
});

verifyProperty(arrObj, "0", {
  value: 101,
  writable: false,
  enumerable: false,
  configurable: false,
});
