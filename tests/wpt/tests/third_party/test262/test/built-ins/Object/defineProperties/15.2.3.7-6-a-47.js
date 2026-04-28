// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-47
description: >
    Object.defineProperties - desc.value and P.value are two numbers
    with the same value (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

var desc = {
  value: 101
};
Object.defineProperty(obj, "foo", desc);

Object.defineProperties(obj, {
  foo: {
    value: 101
  }
});

verifyProperty(obj, "foo", {
  value: 101,
  writable: false,
  enumerable: false,
  configurable: false,
});
