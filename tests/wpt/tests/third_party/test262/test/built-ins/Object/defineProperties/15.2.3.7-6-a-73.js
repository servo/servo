// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-73
description: >
    Object.defineProperties will not throw TypeError if P.configurable
    is false, P.writalbe is false, P.value is undefined and
    properties.value is undefined (8.12.9 step 10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: undefined,
  writable: false,
  configurable: false
});

Object.defineProperties(obj, {
  foo: {
    value: undefined
  }
});

verifyProperty(obj, "foo", {
  value: undefined,
  writable: false,
  enumerable: false,
  configurable: false,
});
