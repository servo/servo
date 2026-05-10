// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-82-6
description: >
    Object.defineProperty - Update [[Enumerable]] attribute of 'name'
    property to true successfully when [[Enumerable]] attribute of
    'name' is false and [[Configurable]] attribute of 'name' is true,
    the 'desc' is a generic descriptor which only contains
    [[Enumerable]] attribute as true, 'name' property is a data
    property (8.12.9 step 8)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: 1001,
  writable: true,
  enumerable: false,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  enumerable: true
});

verifyProperty(obj, "foo", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true,
});
