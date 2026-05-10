// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-291-1
description: >
    Object.defineProperty - 'O' is an Arguments object of a function
    that has formal parameters, 'name' is own accessor property of 'O'
    which is also defined in [[ParameterMap]] of 'O', and 'desc' is
    accessor descriptor, test updating multiple attribute values of
    'name' (10.6 [[DefineOwnProperty]] step 3 and 5.a.i)
includes: [propertyHelper.js]
---*/

(function(a, b, c) {
  function getFunc1() {
    return 10;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc1,
    enumerable: true,
    configurable: true
  });

  function getFunc2() {
    return 20;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc2,
    enumerable: false,
    configurable: false
  });
  if (a !== 0) {
    throw new Test262Error('Expected a === 0, actually ' + a);
  }

  verifyEqualTo(arguments, "0", getFunc2());

  verifyProperty(arguments, "0", {
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
