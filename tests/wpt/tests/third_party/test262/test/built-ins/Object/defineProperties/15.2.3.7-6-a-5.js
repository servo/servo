// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-5
description: >
    Object.defineProperties - 'P' is own accessor property (8.12.9
    step 1 )
includes: [propertyHelper.js]
---*/

var obj = {};

function getFunc() {
  return 11;
}

Object.defineProperty(obj, "prop", {
  get: getFunc,
  configurable: false
});

try {
  Object.defineProperties(obj, {
    prop: {
      value: 12,
      configurable: true
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyEqualTo(obj, "prop", getFunc());

verifyProperty(obj, "prop", {
  enumerable: false,
  configurable: false,
});
