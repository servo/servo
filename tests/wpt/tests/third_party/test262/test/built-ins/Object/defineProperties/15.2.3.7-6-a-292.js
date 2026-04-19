// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-292
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is an
    array index named accessor property of 'O' but not defined in
    [[ParameterMap]] of 'O', and 'desc' is accessor descriptor, test
    updating multiple attribute values of 'P' (10.6
    [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/


var arg;

(function fun() {
  arg = arguments;
}(0, 1, 2));

function get_func1() {
  return 10;
}

Object.defineProperty(arg, "0", {
  get: get_func1,
  enumerable: true,
  configurable: true
});

function get_func2() {
  return 20;
}

Object.defineProperties(arg, {
  "0": {
    get: get_func2,
    enumerable: false,
    configurable: false
  }
});

verifyEqualTo(arg, "0", get_func2());

verifyProperty(arg, "0", {
  enumerable: false,
  configurable: false,
});
