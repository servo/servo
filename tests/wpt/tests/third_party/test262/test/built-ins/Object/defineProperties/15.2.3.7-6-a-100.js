// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-100
description: >
    Object.defineProperties - 'P' is data property, several attributes
    values of P and properties are different (8.12.9 step 12)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: 100,
  writable: true,
  configurable: true
});

Object.defineProperties(obj, {
  foo: {
    value: 200,
    writable: false,
    configurable: false
  }
});

verifyProperty(obj, "foo", {
  value: 200,
  writable: false,
  enumerable: false,
  configurable: false,
});
