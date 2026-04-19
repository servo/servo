// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-50
description: >
    Object.defineProperty - 'name' property doesn't exist in 'O', test
    [[Configurable]] of 'name' property is set as false if it is
    absent in data descriptor 'desc' (8.12.9 step 4.a.i)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  value: 1001,
  writable: true,
  enumerable: true
});

verifyProperty(obj, "property", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: false,
});
