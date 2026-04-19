// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-290-1
description: >
    Object.defineProperty - 'O' is an Arguments object of a function
    that has formal parameters, 'name' is own property which is
    defined in both [[ParameterMap]] of 'O' and 'O', is deleted
    afterwards, and 'desc' is accessor descriptor, test 'name' is
    redefined in 'O' with all correct attribute values (10.6
    [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/

(function(a, b, c) {
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
    enumerable: true,
    configurable: true
  });
  if (a !== 0) {
    throw new Test262Error('Expected a === 0, actually ' + a);
  }

  verifyEqualTo(arguments, "0", getFunc());

  verifyWritable(arguments, "0", "setVerifyHelpProp");

  verifyProperty(arguments, "0", {
    enumerable: true,
    configurable: true,
  });
}(0, 1, 2));
