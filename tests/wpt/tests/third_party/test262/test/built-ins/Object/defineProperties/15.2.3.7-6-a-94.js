// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-94
description: >
    Object.defineProperties - 'P' is data property, properties.value
    and P.value are two different values (8.12.9 step 12)
includes: [propertyHelper.js]
---*/


var obj = {};

obj.foo = 100; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperties(obj, {
  foo: {
    value: 200
  }
});

verifyProperty(obj, "foo", {
  value: 200,
  writable: true,
  enumerable: true,
  configurable: true,
});
