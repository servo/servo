// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-323
description: >
    Object.defineProperty - ''O' is an Arguments object, 'P' is own
    accessor property of 'O', test TypeError is thrown when updating
    the [[Enumerable]] attribute value of 'P' which is not
    configurable (10.6 [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

(function() {
  function setFunc(value) {
    this.genericPropertyString = value;
  }
  Object.defineProperty(arguments, "genericProperty", {
    set: setFunc,
    enumerable: true,
    configurable: false
  });
  try {
    Object.defineProperty(arguments, "genericProperty", {
      enumerable: false
    });
    throw new Test262Error("Expected an exception.");
  } catch (e) {
    verifyWritable(arguments, "genericProperty", "genericPropertyString");

    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }

  verifyProperty(arguments, "genericProperty", {
    enumerable: true,
    configurable: false,
  });
}(1, 2, 3));
