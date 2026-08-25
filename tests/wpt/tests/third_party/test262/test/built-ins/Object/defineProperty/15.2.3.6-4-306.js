// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-306
description: >
    Object.defineProperty - 'O' is an Arguments object, 'name' is an
    index named data property of 'O' but not defined in
    [[ParameterMap]] of 'O', test TypeError is thrown when updating
    the [[Writable]] attribute value of 'name' which is not
    configurable (10.6 [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "0", {
    value: 0,
    writable: false,
    enumerable: false,
    configurable: false
  });
  try {
    Object.defineProperty(arguments, "0", {
      writable: true
    });
    throw new Test262Error("Expected an exception.");
  } catch (e) {
    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }

  verifyProperty(arguments, "0", {
    value: 0,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}());
