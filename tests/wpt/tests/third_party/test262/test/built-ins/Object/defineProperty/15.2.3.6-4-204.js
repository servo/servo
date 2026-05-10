// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-204
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'desc' is data descriptor, test updating all
    attribute values of 'name' (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [1]; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperty(arrObj, "0", {
  value: 1001,
  writable: false,
  enumerable: false,
  configurable: false
});

verifyProperty(arrObj, "0", {
  value: 1001,
  writable: false,
  enumerable: false,
  configurable: false,
});
