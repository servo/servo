// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-253
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is accessor property and 'desc' is accessor
    descriptor, and the [[Configurable]] attribute value of 'name' is
    false, test TypeError is thrown if the [[Set]] field of 'desc' is
    present, and the [[Set]] field of 'desc' is an object and the
    [[Set]] attribute value of 'name' is undefined (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

function getFunc() {
  return 12;
}

Object.defineProperty(arrObj, "1", {
  get: getFunc,
  set: undefined
});

try {
  Object.defineProperty(arrObj, "1", {
    set: function() {}
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyEqualTo(arrObj, "1", getFunc());

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arrObj, "1", {
  enumerable: false,
  configurable: false,
});
