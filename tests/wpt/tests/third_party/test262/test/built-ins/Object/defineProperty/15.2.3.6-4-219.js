// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-219
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    property, test TypeError is thrown when the [[Value]] field of
    'desc' is -0, and the [[Value]] attribute value of 'name' is +0
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, "0", {
  value: +0
});

try {
  Object.defineProperty(arrObj, "0", {
    value: -0
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arrObj, "0", {
  value: +0,
  writable: false,
  enumerable: false,
  configurable: false,
});
