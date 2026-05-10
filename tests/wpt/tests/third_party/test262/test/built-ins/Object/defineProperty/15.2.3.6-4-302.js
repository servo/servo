// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-302
description: >
    Object.defineProperty - 'O' is an Arguments object, 'name' is an
    index named property of 'O' but not defined in [[ParameterMap]] of
    'O', and 'desc' is accessor descriptor, test 'name' is defined in
    'O' with all correct attribute values (10.6 [[DefineOwnProperty]]
    step 3)
includes: [propertyHelper.js]
---*/

(function() {
  delete arguments[0];

  function getFunc() {
    return 10;
  }

  function setFunc(value) {
    this.setVerifyHelpProp = value;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc,
    set: setFunc,
    enumerable: false,
    configurable: false
  });
  verifyEqualTo(arguments, "0", getFunc());

  verifyWritable(arguments, "0", "setVerifyHelpProp");

  verifyProperty(arguments, "0", {
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
