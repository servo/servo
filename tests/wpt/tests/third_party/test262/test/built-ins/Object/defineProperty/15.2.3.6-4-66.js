// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-66
description: >
    Object.defineProperty - desc.value and name.value are two numbers
    with different values (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

obj.foo = 101; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperty(obj, "foo", {
  value: 102
});

verifyProperty(obj, "foo", {
  value: 102,
  writable: true,
  enumerable: true,
  configurable: true,
});
