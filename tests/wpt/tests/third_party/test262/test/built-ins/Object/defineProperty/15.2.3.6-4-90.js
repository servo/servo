// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-90
description: >
    Object.defineProperty will not throw TypeError when
    name.configurable = false, name.writable = false, desc.value and
    name.value are two strings with the same value (8.12.9 step
    10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: "abcd",
  writable: false,
  configurable: false
});

Object.defineProperty(obj, "foo", {
  value: "abcd"
});

verifyProperty(obj, "foo", {
  value: "abcd",
  writable: false,
  enumerable: false,
  configurable: false,
});
