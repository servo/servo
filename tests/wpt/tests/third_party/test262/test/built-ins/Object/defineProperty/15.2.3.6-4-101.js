// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-101
description: >
    Object.defineProperty - 'name' and 'desc' are data properties,
    name.value is present and desc.value is undefined (8.12.9 step 12)
includes: [propertyHelper.js]
---*/

var obj = {};

obj.foo = 100; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperty(obj, "foo", {
  value: undefined
});

verifyProperty(obj, "foo", {
  value: undefined,
  writable: true,
  enumerable: true,
  configurable: true,
});
