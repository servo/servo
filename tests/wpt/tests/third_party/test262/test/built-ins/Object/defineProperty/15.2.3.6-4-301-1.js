// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-301-1
description: >
    Object.defineProperty - 'O' is an Arguments object of a function
    that has formal parameters, 'name' is an index named property of
    'O', and 'desc' is data descriptor, test 'name' is defined in 'O'
    with all correct attribute values (10.6 [[DefineOwnProperty]] step
    3)
includes: [propertyHelper.js]
---*/

(function(a, b, c) {
  delete arguments[0];
  Object.defineProperty(arguments, "0", {
    value: 10,
    writable: false,
    enumerable: false,
    configurable: false
  });
  if (a !== 0) {
    throw new Test262Error("Expected a === 0, actually " + a);
  }

  verifyProperty(arguments, "0", {
    value: 10,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
