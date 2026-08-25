// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-93-1
description: >
    Object.defineProperties will update [[Value]] attribute of named
    data property 'P' successfully when [[Configurable]] attribute is
    true and [[Writable]] attribute is false but not when both are
    false (8.12.9 - step Note & 10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperty(obj, "property", {
  value: 1001,
  writable: false,
  configurable: true
});

Object.defineProperty(obj, "property1", {
  value: 1003,
  writable: false,
  configurable: false
});

try {
  Object.defineProperties(obj, {
    property: {
      value: 1002
    },
    property1: {
      value: 1004
    }
  });

  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "property", {
  value: 1002,
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(obj, "property1", {
  value: 1003,
  writable: false,
  enumerable: false,
  configurable: false,
});
