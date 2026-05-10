// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-299
description: >
    Object.defineProperty - 'O' is an Arguments object, 'name' is own
    accessor property of 'O', test TypeError is thrown when updating
    the [[Enumerable]] attribute value of 'name' which is defined as
    non-configurable (10.6 [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

(function() {
  function getFunc() {
    return 10;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc,
    enumerable: true,
    configurable: false
  });
  try {
    Object.defineProperty(arguments, "0", {
      enumerable: false
    });
    throw new Test262Error("Expected an exception.");
  } catch (e) {
    verifyEqualTo(arguments, "0", getFunc());

    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }

  verifyProperty(arguments, "0", {
    enumerable: true,
    configurable: false,
  });
}(0, 1, 2));
