// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-273
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, name is accessor property and 'desc' is accessor
    descriptor, test updating multiple attribute values of 'name'
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/


var arrObj = [];

function setFunc(value) {
  arrObj.setVerifyHelpProp = value;
}

function getFunc() {
  return 12;
}
Object.defineProperty(arrObj, "1", {
  get: function() {
    return 6;
  },
  set: setFunc,
  enumerable: true,
  configurable: true
});

Object.defineProperty(arrObj, "1", {
  get: getFunc,
  enumerable: false,
  configurable: false
});
verifyEqualTo(arrObj, "1", getFunc());

verifyWritable(arrObj, "1", "setVerifyHelpProp");

verifyProperty(arrObj, "1", {
  enumerable: false,
  configurable: false,
});
