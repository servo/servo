// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-73
description: >
    Object.defineProperty - both desc.writable and name.writable are
    boolean values with the same value (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  writable: false
});

Object.defineProperty(obj, "foo", {
  writable: false
});

verifyProperty(obj, "foo", {
  value: undefined,
  writable: false,
  enumerable: false,
  configurable: false,
});
