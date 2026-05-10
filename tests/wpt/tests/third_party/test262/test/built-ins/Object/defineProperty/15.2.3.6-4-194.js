// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-194
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is own accessor property, test TypeError is
    thrown on updating the configurable attribute from false to true
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];
var getFunc = function() {
  return 11;
};

Object.defineProperty(arrObj, "0", {
  get: getFunc,
  configurable: false
});

try {
  Object.defineProperty(arrObj, "0", {
    configurable: true
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyEqualTo(arrObj, "0", getFunc());

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e.name);
  }
}

verifyProperty(arrObj, "0", {
  enumerable: false,
  configurable: false,
});
