// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-103
description: >
    Object.defineProperty - 'name' and 'desc' are data properties,
    name.writable and desc.writable are different values (8.12.9 step
    12)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  writable: false,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  writable: true
});

verifyProperty(obj, "foo", {
  value: undefined,
  writable: true,
  enumerable: false,
  configurable: true,
});
