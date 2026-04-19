// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-297-1
description: >
    Object.defineProperty - 'O' is an Arguments object of a function
    that has formal parameters, 'name' is own accessor property of 'O'
    which is also defined in [[ParameterMap]] of 'O', test TypeError
    is thrown when updating the [[Get]] attribute value of 'name'
    which is defined as non-configurable (10.6 [[DefineOwnProperty]]
    step 4 and step 5a)
includes: [propertyHelper.js]
---*/

(function(a, b, c) {
  function getFunc1() {
    return 10;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc1,
    enumerable: false,
    configurable: false
  });

  function getFunc2() {
    return 20;
  }
  try {
    Object.defineProperty(arguments, "0", {
      get: getFunc2
    });
    throw new Test262Error("Expected an exception.");
  } catch (e) {
    if (a !== 0) {
      throw new Test262Error('Expected a === 0, actually ' + a);
    }

    verifyEqualTo(arguments, "0", getFunc1());

    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }

  verifyProperty(arguments, "0", {
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
