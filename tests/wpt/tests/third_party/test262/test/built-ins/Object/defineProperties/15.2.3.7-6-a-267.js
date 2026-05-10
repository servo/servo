// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-267
description: >
    Object.defineProperties - 'O' is an Array, 'P' is generic
    property, and 'desc' is accessor descriptor, test 'P' is defined
    in 'O' with all correct attribute values (15.4.5.1 step 5)
includes: [propertyHelper.js]
---*/


var arr = [];

function get_fun() {
  return 12;
}

function set_fun(value) {
  arr.setVerifyHelpProp = value;
}

Object.defineProperties(arr, {
  "property": {
    get: get_fun,
    set: set_fun,
    enumerable: true,
    configurable: true
  }
});
verifyEqualTo(arr, "property", get_fun());

verifyWritable(arr, "property", "setVerifyHelpProp");

verifyProperty(arr, "property", {
  enumerable: true,
  configurable: true,
});

if (arr.length !== 0) {
  throw new Test262Error('Expected arr.length === 0, actually ' + arr.length);
}
