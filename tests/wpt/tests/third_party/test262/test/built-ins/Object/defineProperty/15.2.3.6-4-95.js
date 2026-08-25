// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-95
description: >
    Object.defineProperty will throw TypeError when name.configurable
    = false, name.writable = false, desc.value and name.value are two
    objects which refer to the different objects (8.12.9 step
    10.a.ii.1)
includes: [propertyHelper.js]
---*/


var obj = {};

var obj1 = {
  length: 10
};

Object.defineProperty(obj, "foo", {
  value: obj1,
  writable: false,
  configurable: false
});

var obj2 = {
  length: 20
};

try {
  Object.defineProperty(obj, "foo", {
    value: obj2
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "foo", {
  value: obj1,
  writable: false,
  enumerable: false,
  configurable: false,
});
