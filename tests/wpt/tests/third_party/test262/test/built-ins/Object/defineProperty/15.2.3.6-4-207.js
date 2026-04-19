// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-207
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' property doesn't exist in 'O' and
    [[Enumerable]] is absent in accessor descriptor 'desc', test
    [[Enumerable]] attribute of property 'name' is set to false
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
  configurable: true
});
verifyEqualTo(arrObj, "0", getFunc());

verifyWritable(arrObj, "0", "setVerifyHelpProp");

verifyProperty(arrObj, "0", {
  enumerable: false,
  configurable: true,
});
