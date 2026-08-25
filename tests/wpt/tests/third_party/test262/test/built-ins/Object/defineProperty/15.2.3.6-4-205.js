// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-205
description: >
    Object.defineProperty - 'O' is an Array, 'name' is an array index
    named property, 'name' property doesn't exist in 'O' and [[Get]]
    is absent in accessor descriptor 'desc', test [[Get]] attribute of
    property 'name' is set to undefined (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arrObj = [];
var setFunc = function(value) {
  arrObj.setVerifyHelpProp = value;
};

Object.defineProperty(arrObj, "0", {
  set: setFunc,
  enumerable: true,
  configurable: true
});

verifyWritable(arrObj, "0", "setVerifyHelpProp");

verifyProperty(arrObj, "0", {
  enumerable: true,
  configurable: true,
});
