// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-208
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' property doesn't exist in 'O' and
    [[Configurable]] is absent in accessor descriptor 'desc', test
    [[Configurable]] attribute of property 'name' is set to false
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];
var setFunc = function(value) {
  arrObj.setVerifyHelpProp = value;
};
var getFunc = function() {};

Object.defineProperty(arrObj, "0", {
  set: setFunc,
  get: getFunc,
  enumerable: true
});
verifyEqualTo(arrObj, "0", getFunc());

verifyWritable(arrObj, "0", "setVerifyHelpProp");

verifyProperty(arrObj, "0", {
  enumerable: true,
  configurable: false,
});
