// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-55
description: >
    Object.defineProperty - 'name' property doesn't exist in 'O', test
    [[Enumerable]] of 'name' property of 'Attributes' is set as false
    value if absent in accessor descriptor 'desc' (8.12.9 step 4.b.i)
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
  configurable: true
});
verifyEqualTo(obj, "property", getFunc());

verifyWritable(obj, "property", "setVerifyHelpProp");

verifyProperty(obj, "property", {
  enumerable: false,
  configurable: true,
});
