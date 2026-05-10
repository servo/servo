// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-303
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is
    generic property, and 'desc' is accessor descriptor, test 'P' is
    defined in 'O' with all correct attribute values (10.6
    [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

var arg = (function() {
  return arguments;
}(1, 2, 3));

function getFun() {
  return "getFunctionString";
}

function setFun(value) {
  arg.testGetFunction = value;
}
Object.defineProperties(arg, {
  "genericProperty": {
    get: getFun,
    set: setFun,
    enumerable: true,
    configurable: true
  }
});

verifyEqualTo(arg, "genericProperty", getFun());

verifyWritable(arg, "genericProperty", "testGetFunction");

verifyProperty(arg, "genericProperty", {
  enumerable: true,
  configurable: true,
});
