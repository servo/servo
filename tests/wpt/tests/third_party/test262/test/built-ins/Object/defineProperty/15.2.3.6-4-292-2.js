// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-292-1
description: >
    Object.defineProperty - 'O' is an Arguments object of a function
    that has formal parameters, 'name' is own property of 'O' which is
    also defined in [[ParameterMap]] of 'O', and 'desc' is data
    descriptor, test updating multiple attribute values of 'name'
    (10.6 [[DefineOwnProperty]] step 3 and 5.b)
includes: [propertyHelper.js]
flags: [onlyStrict]
---*/

(function(a, b, c) {
  Object.defineProperty(arguments, "0", {
    value: 20,
    writable: false,
    enumerable: false,
    configurable: false
  });

  if (a !== 0) {
    throw new Test262Error('Expected a === 0, actually ' + a);
  }

  verifyProperty(arguments, "0", {
    value: 20,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
