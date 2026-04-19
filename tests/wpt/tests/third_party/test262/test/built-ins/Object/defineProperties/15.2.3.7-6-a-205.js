// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-205
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'desc' is accessor descriptor, test updating all
    attribute values of 'P'  (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];

Object.defineProperties(arr, {
  "0": {
    get: function() {
      return 11;
    },
    set: function() {},
    configurable: true,
    enumerable: true
  }
});

var setFun = function(value) {
  arr.setVerifyHelpProp = value;
};
var getFun = function() {
  return 14;
};
Object.defineProperties(arr, {
  "0": {
    get: getFun,
    set: setFun,
    configurable: false,
    enumerable: false
  }
});

verifyEqualTo(arr, "0", getFun());

verifyWritable(arr, "0", "setVerifyHelpProp");

verifyProperty(arr, "0", {
  enumerable: false,
  configurable: false,
});
