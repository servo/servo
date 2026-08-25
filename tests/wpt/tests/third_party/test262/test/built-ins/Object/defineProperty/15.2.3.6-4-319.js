// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-319
description: >
    Object.defineProperty - 'O' is an Arguments object, 'P' is own
    data property of 'O', test TypeError is thrown when updating the
    [[Enumerable]] attribute value of 'P' which is not configurable
    (10.6 [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "genericProperty", {
    enumerable: true,
    configurable: false
  });
  try {
    Object.defineProperty(arguments, "genericProperty", {
      enumerable: false
    });
    throw new Test262Error("Expected an exception.");
  } catch (e) {
    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }

  verifyProperty(arguments, "genericProperty", {
    value: undefined,
    writable: false,
    enumerable: true,
    configurable: false,
  });
}(1, 2, 3));
