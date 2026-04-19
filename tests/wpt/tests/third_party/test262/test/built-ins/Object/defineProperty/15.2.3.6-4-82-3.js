// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-82-3
description: >
    Object.defineProperty - Update [[Configurable]] attribute of
    'name' property to false successfully when [[Configurable]]
    attribute of 'name' property is true,  the 'desc' is a generic
    descriptor which contains [[Configurable]] attribute as false,
    'name' property is a data property (8.12.9 step 8)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  configurable: false
});

verifyProperty(obj, "foo", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: false,
});
