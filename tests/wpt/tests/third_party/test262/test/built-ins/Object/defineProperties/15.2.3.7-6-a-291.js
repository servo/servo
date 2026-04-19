// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-291
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is an
    array index named property of 'O' but not defined in
    [[ParameterMap]] of 'O', and 'desc' is accessor descriptor, test
    'P' is defined in 'O' with all correct attribute values (10.6
    [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/


var arg;

(function fun() {
  arg = arguments;
}(0, 1, 2));

delete arg[0];

function get_func() {
  return 10;
}

function set_func(value) {
  arg.setVerifyHelpProp = value;
}

Object.defineProperties(arg, {
  "0": {
    get: get_func,
    set: set_func,
    enumerable: false,
    configurable: false
  }
});

verifyEqualTo(arg, "0", get_func());

verifyWritable(arg, "0", "setVerifyHelpProp");

verifyProperty(arg, "0", {
  enumerable: false,
  configurable: false,
});
