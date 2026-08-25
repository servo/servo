// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-293
description: >
    Object.defineProperty - 'O' is an Arguments object, 'name' is own
    data property of 'O', test TypeError is thrown when updating the
    [[Value]] attribute value of 'name' which is defined as
    non-writable and non-configurable (10.6 [[DefineOwnProperty]] step
    3)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "0", {
    value: 10,
    writable: false,
    enumerable: false,
    configurable: false
  });
  try {
    Object.defineProperty(arguments, "0", {
      value: 20
    });
    throw new Test262Error("Expected an exception.");
  } catch (e) {
    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }

  verifyProperty(arguments, "0", {
    value: 10,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
