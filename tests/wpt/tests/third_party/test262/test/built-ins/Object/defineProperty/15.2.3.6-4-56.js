// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-56
description: >
    Object.defineProperty - 'name' property doesn't exist in 'O', test
    [[Configurable]] of 'name' property is set as false if it is
    absent in accessor descriptor 'desc' (8.12.9 step 4.b.i)
includes: [propertyHelper.js]
---*/

var obj = {};
var setFunc = function(value) {
  obj.setVerifyHelpProp = value;
};
var getFunc = function() {
  return 10;
};

Object.defineProperty(obj, "property", {
  set: setFunc,
  get: getFunc,
  enumerable: true
});
verifyEqualTo(obj, "property", getFunc());

verifyWritable(obj, "property", "setVerifyHelpProp");

verifyProperty(obj, "property", {
  enumerable: true,
  configurable: false,
});
