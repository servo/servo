// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-65
description: >
    Object.defineProperties throws TypeError when P.configurable is
    false and desc.configurable is true (8.12.9 step 7.a)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: 10,
  configurable: false
});

try {
  Object.defineProperties(obj, {
    foo: {
      configurable: true
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "foo", {
  value: 10,
  writable: false,
  enumerable: false,
  configurable: false,
});
