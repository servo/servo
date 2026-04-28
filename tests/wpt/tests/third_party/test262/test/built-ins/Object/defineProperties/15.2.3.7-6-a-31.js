// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-31
description: >
    Object.defineProperties - 'desc' is data descriptor, test setting
    all attribute values of 'P' (8.12.9 step 4.a.i)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperties(obj, {
  prop: {
    value: 1002,
    writable: false,
    enumerable: false,
    configurable: false
  }
});

verifyProperty(obj, "prop", {
  value: 1002,
  writable: false,
  enumerable: false,
  configurable: false,
});
