// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-257
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' is accessor property and 'desc' is accessor
    descriptor, and the [[Configurable]] attribute value of 'name' is
    false, test TypeError is not thrown if the [[Get]] field of 'desc'
    is present, and the [[Get]] field of 'desc' and the [[Get]]
    attribute value of 'name' are undefined (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];

function setFunc(value) {
  arrObj.setVerifyHelpProp = value;
}

Object.defineProperty(arrObj, "1", {
  get: undefined,
  set: setFunc,
  configurable: false
});

Object.defineProperty(arrObj, "1", {
  get: undefined
});

verifyWritable(arrObj, "1", "setVerifyHelpProp");

verifyProperty(arrObj, "1", {
  enumerable: false,
  configurable: false,
});
