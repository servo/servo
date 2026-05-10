// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-316
description: >
    Object.defineProperty - 'O' is an Arguments object, 'P' is generic
    own data property of 'O', and 'desc' is data descriptor, test
    updating multiple attribute values of 'P' (10.6
    [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "genericProperty", {
    value: 1001,
    writable: true,
    enumerable: true,
    configurable: true
  });
  Object.defineProperty(arguments, "genericProperty", {
    value: 1002,
    enumerable: false,
    configurable: false
  });

  verifyProperty(arguments, "genericProperty", {
    value: 1002,
    writable: true,
    enumerable: false,
    configurable: false,
  });
}(1, 2, 3));
