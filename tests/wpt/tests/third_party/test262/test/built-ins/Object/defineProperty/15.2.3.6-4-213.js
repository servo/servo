// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-213
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' makes no change if the value of every field
    in 'desc' is the same value as the corresponding field in
    'name'(desc is accessor property) (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];
var getFunc = function() {
  return "100";
};
var setFunc = function(value) {
  arrObj.setVerifyHelpProp = value;
};

var desc = {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: true
};

Object.defineProperty(arrObj, "0", {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: true
});

Object.defineProperty(arrObj, "0", desc);

verifyEqualTo(arrObj, "0", getFunc());

verifyWritable(arrObj, "0", "setVerifyHelpProp");

verifyProperty(arrObj, "0", {
  enumerable: true,
  configurable: true,
});
