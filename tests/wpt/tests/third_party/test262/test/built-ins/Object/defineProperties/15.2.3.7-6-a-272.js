// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-272
description: >
    Object.defineProperties - 'O' is an Array, 'P' is generic own data
    property of 'O', test TypeError is thrown when updating the
    [[Enumerable]] attribute value of 'P' which is defined as
    non-configurable (15.4.5.1 step 5)
includes: [propertyHelper.js]
---*/


var arr = [];

Object.defineProperty(arr, "property", {
  value: 12,
  enumerable: false
});

try {
  Object.defineProperties(arr, {
    "property": {
      enumerable: true
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arr, "property", {
  value: 12,
  writable: false,
  enumerable: false,
  configurable: false,
});
