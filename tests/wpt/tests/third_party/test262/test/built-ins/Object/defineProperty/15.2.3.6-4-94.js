// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-94
description: >
    Object.defineProperty will not throw TypeError when
    name.configurable = false, name.writable = false, desc.value and
    name.value are two Objects refer to the same object (8.12.9 step
    10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

var obj1 = {
  length: 10
};

Object.defineProperty(obj, "foo", {
  value: obj1,
  writable: false,
  configurable: false
});

Object.defineProperty(obj, "foo", {
  value: obj1
});

verifyProperty(obj, "foo", {
  value: obj1,
  writable: false,
  enumerable: false,
  configurable: false,
});
