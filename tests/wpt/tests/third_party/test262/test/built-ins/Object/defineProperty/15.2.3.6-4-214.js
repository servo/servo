// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-214
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    property and its configurable and writable attributes are set to
    false, test TypeError is thrown when the type of the [[Value]]
    field of 'desc' is different from the type of the [[Value]]
    attribute value of 'name' (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

Object.defineProperty(arrObj, 0, {
  value: 101,
  writable: false,
  configurable: false
});

try {
  Object.defineProperty(arrObj, "0", {
    value: "abc"
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arrObj, "0", {
  value: 101,
  writable: false,
  enumerable: false,
  configurable: false,
});
