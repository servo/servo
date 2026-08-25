// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-52
description: >
    Object.defineProperty - 'desc' is generic descriptor without any
    attribute, test 'name' is defined in 'obj' with all default
    attribute values (8.12.9 step 4.a.i)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "property", {});

verifyProperty(obj, "property", {
  value: undefined,
  writable: false,
  enumerable: false,
  configurable: false,
});
