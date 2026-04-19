// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-313
description: >
    Object.defineProperty - 'O' is an Arguments object, 'P' is generic
    property, and 'desc' is data descriptor, test 'P' is defined in
    'O' with all correct attribute values (10.6 [[DefineOwnProperty]]
    step 3)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "genericProperty", {
    value: 1001,
    writable: true,
    enumerable: true,
    configurable: true
  });

  verifyProperty(arguments, "genericProperty", {
    value: 1001,
    writable: true,
    enumerable: true,
    configurable: true,
  });
}(1, 2, 3));
