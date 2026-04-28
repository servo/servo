// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-294-1
description: >
    Object.defineProperty - 'O' is an Arguments object of a function
    that has formal parameters, 'name' is own data property of 'O'
    which is also defined in [[ParameterMap]] of 'O', test TypeError
    is thrown when updating the [[Writable]] attribute value of 'name'
    which is defined as non-configurable (10.6 [[DefineOwnProperty]]
    step 4 and 5b)
includes: [propertyHelper.js]
flags: [noStrict]
---*/

(function(a, b, c) {
  Object.defineProperty(arguments, "0", {
    value: 10,
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

    if (a !== 10) {
      throw new Test262Error('Expected "a === 10", actually ' + a);
    }
  }

  verifyProperty(arguments, "0", {
    value: 10,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
