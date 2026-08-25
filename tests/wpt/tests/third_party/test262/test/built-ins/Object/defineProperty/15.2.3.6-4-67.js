// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-67
description: >
    Object.defineProperty - both desc.value and name.value are two
    strings which have same length and same characters in
    corresponding positions (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: "abcd"
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
