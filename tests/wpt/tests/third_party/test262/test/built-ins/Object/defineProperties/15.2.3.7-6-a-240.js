// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-240
description: >
    Object.defineProperties - TypeError is thrown if 'O' is an Array,
    'P' is an array index named property that already exists on 'O' is
    data property with  [[Configurable]], [[Writable]] false, 'desc'
    is data descriptor, [[Value]] field of 'desc' and the [[Value]]
    attribute value of 'P' are two objects which refer to the
    different objects (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];
var obj1 = {
  value: 12
};
var obj2 = {
  value: 36
};

Object.defineProperty(arr, "1", {
  value: obj1
});

try {
  Object.defineProperties(arr, {
    "1": {
      value: obj2
    }
  });

  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arr, "1", {
  value: obj1,
  writable: false,
  enumerable: false,
  configurable: false,
});
