// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-238
description: >
    Object.defineProperties - TypeError is thrown if 'O' is an Array,
    'P' is an array index named property that already exists on 'O' is
    data property with  [[Configurable]], [[Writable]] false, 'desc'
    is data descriptor, [[Value]] field of 'desc' and the [[Value]]
    attribute value of 'P' are two strings with different values
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];

Object.defineProperty(arr, "1", {
  value: "abcd"
});

try {
  Object.defineProperties(arr, {
    "1": {
      value: "efgh"
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arr, "1", {
  value: "abcd",
  writable: false,
  enumerable: false,
  configurable: false,
});
