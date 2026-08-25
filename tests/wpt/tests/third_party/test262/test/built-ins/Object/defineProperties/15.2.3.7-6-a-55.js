// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-55
description: >
    Object.defineProperties - both desc.writable and P.writable are
    boolean values with the same value (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

var desc = {
  writable: false
};
Object.defineProperty(obj, "foo", desc);

Object.defineProperties(obj, {
  foo: {
    writable: false
  }
});

verifyProperty(obj, "foo", {
  value: undefined,
  writable: false,
  enumerable: false,
  configurable: false,
});
