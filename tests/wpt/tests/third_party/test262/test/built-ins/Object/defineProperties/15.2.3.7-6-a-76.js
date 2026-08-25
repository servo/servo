// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-76
description: >
    Object.defineProperties throws TypeError when P.configurable is
    false, P.writalbe is false, properties.value is +0 and P.value is
    -0 (8.12.9 step 10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: -0,
  writable: false,
  configurable: false
});

try {
  Object.defineProperties(obj, {
    foo: {
      value: +0
    }
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
