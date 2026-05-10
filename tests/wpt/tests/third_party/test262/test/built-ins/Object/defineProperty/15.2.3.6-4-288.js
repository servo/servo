// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-288
description: >
    Object.defineProperty - 'O' is an Array, 'name' is generic own
    accessor property of 'O', test TypeError is thrown when updating
    the [[Configurable]] attribute value of 'name' which is defined as
    non-configurable (15.4.5.1 step 5)
includes: [propertyHelper.js]
---*/


var arrObj = [];

function setFunc(value) {
  arrObj.setVerifyHelpProp = value;
}
Object.defineProperty(arrObj, "property", {
  set: setFunc,
  configurable: false
});
try {
  Object.defineProperty(arrObj, "property", {
    configurable: true
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyWritable(arrObj, "property", "setVerifyHelpProp");

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arrObj, "property", {
  enumerable: false,
  configurable: false,
});
