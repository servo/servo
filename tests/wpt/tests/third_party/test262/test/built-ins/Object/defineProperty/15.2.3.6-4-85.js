// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-85
description: >
    Object.defineProperty will not throw TypeError if
    name.configurable = false, name.writable = false, name.value = NaN
    and desc.value = NaN (8.12.9 step 10.a.ii.1)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: NaN,
  writable: false,
  configurable: false
});

Object.defineProperty(obj, "foo", {
  value: NaN,
  writable: false,
  configurable: false
});

verifyProperty(obj, "foo", {
  value: NaN,
  writable: false,
  enumerable: false,
  configurable: false,
});
