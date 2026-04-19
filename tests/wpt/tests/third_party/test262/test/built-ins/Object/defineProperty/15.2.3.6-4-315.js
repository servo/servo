// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-315
description: >
    Object.defineProperty - 'O' is an Arguments object, 'P' is generic
    own accessor property of 'O', and 'desc' is accessor descriptor,
    test updating multiple attribute values of 'P' (10.6
    [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "genericProperty", {
    get: function() {
      return 1001;
    },
    set: function(value) {
      this.testgetFunction1 = value;
    },
    enumerable: true,
    configurable: true
  });

  function getFunc() {
    return "getFunctionString";
  }

  function setFunc(value) {
    this.testgetFunction = value;
  }
  Object.defineProperty(arguments, "genericProperty", {
    get: getFunc,
    set: setFunc,
    enumerable: false,
    configurable: false
  });
  verifyEqualTo(arguments, "genericProperty", getFunc());

  verifyWritable(arguments, "genericProperty", "testgetFunction");

  verifyProperty(arguments, "genericProperty", {
    enumerable: false,
    configurable: false,
  });
}(1, 2, 3));
