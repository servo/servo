// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-64
description: >
    Object.defineProperty - desc.value = +0 and name.value = -0
    (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: -0
});

try {
  Object.defineProperty(obj, "foo", {
    value: +0
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "foo", {
  value: -0,
  writable: false,
  enumerable: false,
  configurable: false,
});
