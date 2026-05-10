// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-70
description: >
    Object.defineProperty - desc.value and name.value are two boolean
    values with different values (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

obj.foo = true; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperty(obj, "foo", {
  value: false
});

verifyProperty(obj, "foo", {
  value: false,
  writable: true,
  enumerable: true,
  configurable: true,
});
