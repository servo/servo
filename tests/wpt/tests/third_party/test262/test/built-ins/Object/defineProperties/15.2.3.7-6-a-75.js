// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-75
description: >
    Object.defineProperties will not throw TypeError if P.configurable
    is false, P.writalbe is false, P.value is NaN and properties.value
    is NaN (8.12.9 step 10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};
var accessed = false;

Object.defineProperty(obj, "foo", {
  value: NaN,
  writable: false,
  configurable: false
});

Object.defineProperties(obj, {
  foo: {
    value: NaN
  }
});

verifyProperty(obj, "foo", {
  writable: false,
  enumerable: false,
  configurable: false,
});
