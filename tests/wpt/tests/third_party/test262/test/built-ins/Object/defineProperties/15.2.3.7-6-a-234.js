// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-234
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    property, 'P' is data property and 'desc' is data descriptor, and
    the [[Configurable]] attribute value of 'P' is false, test
    TypeError is thrown if the [[Writable]] attribute value of 'P' is
    false, and the type of the [[Value]] field of 'desc' is different
    from the type of the [[Value]] attribute value of 'P'  (15.4.5.1
    step 4.c)
includes: [propertyHelper.js]
---*/


var arr = [];

Object.defineProperty(arr, "1", {
  value: 3,
  configurable: false,
  writable: false
});

try {

  Object.defineProperties(arr, {
    "1": {
      value: "abc"
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arr, "1", {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: false,
});
