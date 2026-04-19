// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-57
description: >
    Object.defineProperty - 'desc' is accessor descriptor, test
    updating all attribute values of 'name' (8.12.9 step 4.b.i)
includes: [propertyHelper.js]
---*/

var obj = {};
var setFunc = function(value) {
  obj.setVerifyHelpProp = value;
};
var getFunc = function() {
  return 14;
};

Object.defineProperty(obj, "property", {
  get: function() {
    return 11;
  },
  set: function(value) {},
  configurable: true,
  enumerable: true
});

Object.defineProperty(obj, "property", {
  get: getFunc,
  set: setFunc,
  configurable: false,
  enumerable: false
});

verifyEqualTo(obj, "property", getFunc());

verifyWritable(obj, "property", "setVerifyHelpProp");

verifyProperty(obj, "property", {
  enumerable: false,
  configurable: false,
});
