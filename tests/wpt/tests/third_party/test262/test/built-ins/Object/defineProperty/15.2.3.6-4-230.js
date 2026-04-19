// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-230
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    property, the [[Get]] field of 'desc' and the [[Get]] attribute
    value of 'name' are two objects which refer to the same object
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];
arrObj.helpVerifySet = 10;

function getFunc() {
  return arrObj.helpVerifySet;
}

function setFunc(value) {
  arrObj.helpVerifySet = value;
}

Object.defineProperty(arrObj, "0", {
  get: getFunc,
  set: setFunc
});

Object.defineProperty(arrObj, "0", {
  get: getFunc
});
verifyEqualTo(arrObj, "0", getFunc());

verifyWritable(arrObj, "0", "helpVerifySet");

verifyProperty(arrObj, "0", {
  enumerable: false,
  configurable: false,
});
