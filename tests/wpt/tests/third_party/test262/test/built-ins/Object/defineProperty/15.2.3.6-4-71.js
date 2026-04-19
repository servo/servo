// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-71
description: >
    Object.defineProperty - both desc.value and name.value are Ojbects
    which refer to the same Object (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

var obj1 = {
  length: 10
};

Object.defineProperty(obj, "foo", {
  value: obj1
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
