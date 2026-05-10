// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-83
description: >
    Object.defineProperty will not throw TypeError if
    name.configurable = false, name.writable = false, name.value =
    undefined and desc.value = undefined (8.12.9 step 10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: undefined,
  writable: false,
  configurable: false
});

Object.defineProperty(obj, "foo", {
  value: undefined,
  writable: false,
  configurable: false
});

verifyProperty(obj, "foo", {
  value: undefined,
  writable: false,
  enumerable: false,
  configurable: false,
});
