// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-102
description: >
    Object.defineProperty - 'name' and 'desc' are data properties,
    desc.value is present and name.value is undefined (8.12.9 step 12)
includes: [propertyHelper.js]
---*/

var obj = {};

obj.foo = undefined; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperty(obj, "foo", {
  value: 100
});

verifyProperty(obj, "foo", {
  value: 100,
  writable: true,
  enumerable: true,
  configurable: true,
});
