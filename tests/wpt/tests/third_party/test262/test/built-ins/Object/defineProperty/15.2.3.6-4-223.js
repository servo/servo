// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-223
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    property, test TypeError is thrown  when the [[Value]] field of
    'desc' and the [[Value]] attribute value of 'name' are two strings
    with different values (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, 0, {
  value: "abcd",
  writable: false,
  configurable: false
});

try {
  Object.defineProperty(arrObj, "0", {
    value: "fghj"
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arrObj, "0", {
  value: "abcd",
  writable: false,
  enumerable: false,
  configurable: false,
});
