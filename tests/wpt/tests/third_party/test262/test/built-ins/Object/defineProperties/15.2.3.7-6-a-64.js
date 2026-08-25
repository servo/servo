// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-64
description: >
    Object.defineProperties - desc.configurable and P.configurable are
    two boolean values with different values (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: 10,
  configurable: true
});

Object.defineProperties(obj, {
  foo: {
    configurable: false
  }
});

verifyProperty(obj, "foo", {
  value: 10,
  writable: false,
  enumerable: false,
  configurable: false,
});
