// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-82-15
description: >
    Object.defineProperty - Update [[Configurable]] attribute of
    'name' property to false successfully when [[Configurable]]
    attribute of 'name' property is true,  the 'desc' is a generic
    descriptor which contains [[Configurable]] attribute as false,
    'name' property is an index data property (8.12.9 step 8)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "0", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "0", {
  configurable: false
});

verifyProperty(obj, "0", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: false,
});
