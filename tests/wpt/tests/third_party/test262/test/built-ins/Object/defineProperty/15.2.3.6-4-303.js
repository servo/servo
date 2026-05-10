// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-303
description: >
    Object.defineProperty - 'O' is an Arguments object, 'name' is an
    index named accessor property of 'O' but not defined in
    [[ParameterMap]] of 'O', and 'desc' is accessor descriptor, test
    updating multiple attribute values of 'name' (10.6
    [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/

(function() {
  function getFunc1() {
    return 10;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc1,
    enumerable: true,
    configurable: true
  });

  function getFunc2() {
    return 20;
  }
  Object.defineProperty(arguments, "0", {
    get: getFunc2,
    enumerable: false,
    configurable: false
  });
  verifyEqualTo(arguments, "0", getFunc2());

  verifyProperty(arguments, "0", {
    enumerable: false,
    configurable: false,
  });
}());
