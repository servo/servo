// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-285
description: >
    Object.defineProperty - 'O' is an Array, 'name' is generic own
    accessor property of 'O', test TypeError is thrown when updating
    the [[Get]] attribute value of 'name' which is defined as
    non-configurable (15.4.5.1 step 5)
includes: [propertyHelper.js]
---*/


var arrObj = [];

function getFunc() {
  return 12;
}

function setFunc(value) {
  arrObj.setVerifyHelpProp = value;
}
Object.defineProperty(arrObj, "property", {
  get: getFunc,
  set: setFunc
});
try {
  Object.defineProperty(arrObj, "property", {
    get: function() {
      return 36;
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyEqualTo(arrObj, "property", getFunc());

  verifyWritable(arrObj, "property", "setVerifyHelpProp");

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arrObj, "property", {
  enumerable: false,
  configurable: false,
});
