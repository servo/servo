// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-300-1
description: >
    Object.defineProperty - 'O' is an Arguments object of a function
    that has formal parameters, 'name' is own accessor property of 'O'
    which is also defined in [[ParameterMap]] of 'O', test TypeError
    is thrown when updating the [[Configurable]] attribute value of
    'name' which is defined as non-configurable (10.6
    [[DefineOwnProperty]] step 4 and step 5a)
includes: [propertyHelper.js]
---*/

(function(a, b, c) {
  function getFunc() {
    return 0;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc,
    enumerable: true,
    configurable: false
  });
  try {
    Object.defineProperty(arguments, "0", {
      configurable: true
    });
    throw new Test262Error("Expected an exception.");
  } catch (e) {
    if (a !== 0) {
      throw new Test262Error('Expected a === 0, actually ' + a);
    }
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
