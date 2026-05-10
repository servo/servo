// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-278
description: >
    Object.defineProperty - 'name' is generic property that won't
    exist on 'O', and 'desc' is accessor descriptor, test 'name' is
    defined in 'O' with all correct attribute values (15.4.5.1 step 5)
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
  set: setFunc,
  enumerable: true,
  configurable: true
});

verifyEqualTo(arrObj, "property", getFunc());

verifyWritable(arrObj, "property", "setVerifyHelpProp");

verifyProperty(arrObj, "property", {
  enumerable: true,
  configurable: true,
});
