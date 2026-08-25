// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-83
description: >
    Object.defineProperties throws TypeError when P.configurable is
    false, P.writalbe is false, properties.value and P.value are two
    booleans with different values (8.12.9 step 10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "foo", {
  value: false,
  writable: false,
  configurable: false
});

try {
  Object.defineProperties(obj, {
    foo: {
      value: true
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "foo", {
  value: false,
  writable: false,
  enumerable: false,
  configurable: false,
});
