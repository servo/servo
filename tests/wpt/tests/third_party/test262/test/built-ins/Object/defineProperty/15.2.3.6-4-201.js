// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-201
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' property doesn't exist in 'O' and
    [[Writable]] is absent in data descriptor 'desc', test
    [[Writable]] attribute of  property 'name' is set to false
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, "0", {
  value: 1001,
  enumerable: true,
  configurable: false
});

verifyProperty(arrObj, "0", {
  value: 1001,
  writable: false,
  enumerable: true,
  configurable: false,
});
