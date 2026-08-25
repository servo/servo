// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-97
description: >
    Object.defineProperty will throw TypeError when name.configurable
    = false, name.[[Set]] is undefined, desc.[[Set]] refers to an
    object (8.12.9 step 11.a.i)
includes: [propertyHelper.js]
---*/


var obj = {};

function getFunc() {
  return "property";
}

Object.defineProperty(obj, "property", {
  get: getFunc,
  configurable: false
});

try {
  Object.defineProperty(obj, "property", {
    get: getFunc,
    set: function() {},
    configurable: false
  });

  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyEqualTo(obj, "property", getFunc());

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "property", {
  enumerable: false,
  configurable: false,
});
