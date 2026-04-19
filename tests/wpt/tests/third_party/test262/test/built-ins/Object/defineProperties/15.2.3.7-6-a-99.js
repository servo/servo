// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-99
description: >
    Object.defineProperties - 'P' is data property, P.configurable is
    true and properties.configurable is false
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: 200,
  enumerable: true,
  writable: true,
  configurable: true
});

Object.defineProperties(obj, {
  foo: {
    configurable: false
  }
});

verifyProperty(obj, "foo", {
  value: 200,
  writable: true,
  enumerable: true,
  configurable: false,
});
