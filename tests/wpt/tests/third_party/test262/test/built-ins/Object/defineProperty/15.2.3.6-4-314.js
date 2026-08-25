// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-314
description: >
    Object.defineProperty - 'O' is an Arguments object, 'P' is generic
    property, and 'desc' is accessor descriptor, test 'P' is defined
    in 'O' with all correct attribute values (10.6
    [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/

(function() {
  function getFunc() {
    return "getFunctionString";
  }

  function setFunc(value) {
    this.testgetFunction = value;
  }
  Object.defineProperty(arguments, "genericProperty", {
    get: getFunc,
    set: setFunc,
    enumerable: true,
    configurable: true
  });
  verifyEqualTo(arguments, "genericProperty", getFunc());

  verifyWritable(arguments, "genericProperty", "testgetFunction");

  verifyProperty(arguments, "genericProperty", {
    enumerable: true,
    configurable: true,
  });
}(1, 2, 3));
