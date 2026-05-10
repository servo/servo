// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-98
description: >
    Object.defineProperties - 'P' is data property, P.enumerable and
    properties.enumerable are different values (8.12.9 step 12)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: 200,
  enumerable: false,
  writable: true,
  configurable: true
});

Object.defineProperties(obj, {
  foo: {
    enumerable: true
  }
});

verifyProperty(obj, "foo", {
  value: 200,
  writable: true,
  enumerable: true,
  configurable: true,
});
