// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-243
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property,  'name' is accessor property and 'desc' is data
    descriptor, and the [[Configurable]] attribute value of 'name' is
    true, test 'name' is converted from accessor property to data
    property (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/


var arrObj = [];

function getFunc() {
  return 3;
}
Object.defineProperty(arrObj, "1", {
  get: getFunc,
  configurable: true
});

Object.defineProperty(arrObj, "1", {
  value: 12
});

verifyProperty(arrObj, "1", {
  value: 12,
  writable: false,
  enumerable: false,
  configurable: true,
});
