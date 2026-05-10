// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-49
description: >
    Object.defineProperties - both desc.value and P.value are two
    strings which have same length and same characters in
    corresponding positions (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

var desc = {
  value: "abcd"
};
Object.defineProperty(obj, "foo", desc);

Object.defineProperties(obj, {
  foo: {
    value: "abcd"
  }
});

verifyProperty(obj, "foo", {
  value: "abcd",
  writable: false,
  enumerable: false,
  configurable: false,
});
