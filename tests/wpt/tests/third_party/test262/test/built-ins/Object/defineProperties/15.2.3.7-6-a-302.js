// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-302
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is
    generic property, and 'desc' is data descriptor, test 'P' is
    defined in 'O' with all correct attribute values (10.6
    [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

var arg = (function() {
  return arguments;
}(1, 2, 3));

Object.defineProperties(arg, {
  "genericProperty": {
    value: 1001,
    writable: true,
    enumerable: true,
    configurable: true
  }
});

verifyProperty(arg, "genericProperty", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true,
});
