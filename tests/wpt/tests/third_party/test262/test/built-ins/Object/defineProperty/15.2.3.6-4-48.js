// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-48
description: >
    Object.defineProperty - 'name' property doesn't exist in 'O', test
    [[Writable]] of 'name' property of 'Attributes' is set as false
    value if absent in data descriptor 'desc' (8.12.9 step 4.a.i)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  value: 1001,
  enumerable: true,
  configurable: false
});

verifyProperty(obj, "property", {
  value: 1001,
  writable: false,
  enumerable: true,
  configurable: false,
});
