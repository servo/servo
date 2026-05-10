// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-36
description: >
    Object.defineProperties - 'P' doesn't exist in 'O', test
    [[Configurable]] of 'P' is set as false value if absent in
    accessor descriptor 'desc' (8.12.9 step 4.b.i)
includes: [propertyHelper.js]
---*/

var obj = {};
var getFun = function() {
  return 10;
};
var setFun = function(value) {
  obj.setVerifyHelpProp = value;
};

Object.defineProperties(obj, {
  prop: {
    set: setFun,
    get: getFun,
    enumerable: true
  }
});

verifyEqualTo(obj, "prop", 10);

verifyWritable(obj, "prop", "setVerifyHelpProp");

verifyProperty(obj, "prop", {
  enumerable: true,
  configurable: false,
});
