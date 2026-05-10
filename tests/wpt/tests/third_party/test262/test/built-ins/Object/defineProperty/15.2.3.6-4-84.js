// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-84
description: >
    Object.defineProperty will not throw TypeError if
    name.configurable = false, name.writable = false, name.value =
    null and desc.value = null (8.12.9 step 10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: null,
  writable: false,
  configurable: false
});

Object.defineProperty(obj, "foo", {
  value: null,
  writable: false,
  configurable: false
});

verifyProperty(obj, "foo", {
  value: null,
  writable: false,
  enumerable: false,
  configurable: false,
});
