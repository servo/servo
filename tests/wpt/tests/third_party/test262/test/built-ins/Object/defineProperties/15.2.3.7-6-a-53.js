// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-53
description: >
    Object.defineProperties - both desc.value and P.value are Ojbects
    which refer to the same Object (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

var obj1 = {
  length: 10
};
var desc = {
  value: obj1
};
Object.defineProperty(obj, "foo", desc);

Object.defineProperties(obj, {
  foo: {
    value: obj1
  }
});

verifyProperty(obj, "foo", {
  value: obj1,
  writable: false,
  enumerable: false,
  configurable: false,
});
