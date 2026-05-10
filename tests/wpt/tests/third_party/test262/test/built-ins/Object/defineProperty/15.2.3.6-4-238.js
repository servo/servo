// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-238
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, TypeError is thrown if the [[Configurable]]
    attribute value of 'name' is false  and the [[Configurable]] field
    of 'desc' is true (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/


var arrObj = [];

Object.defineProperty(arrObj, "1", {
  value: 3,
  writable: true,
  configurable: false
});

try {
  Object.defineProperty(arrObj, "1", {
    value: 13,
    writable: true,
    configurable: true
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arrObj, "1", {
  value: 3,
  writable: true,
  enumerable: false,
  configurable: false,
});
