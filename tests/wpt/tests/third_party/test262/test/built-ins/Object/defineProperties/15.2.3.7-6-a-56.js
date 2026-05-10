// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-56
description: >
    Object.defineProperties - desc.writable and P.writable are two
    boolean values with different values (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

var desc = {
  writable: false,
  configurable: true
};
Object.defineProperty(obj, "foo", desc);

Object.defineProperties(obj, {
  foo: {
    writable: true,
    configurable: true
  }
});

verifyProperty(obj, "foo", {
  value: undefined,
  writable: true,
  enumerable: false,
  configurable: true,
});
