// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-90
description: >
    Object.defineProperties will not throw TypeError when
    P.configurable is false, both properties.[[Get]] and P.[[Get]] are
    two objects which refer to the same object (8.12.9 step 11.a.ii)
includes: [propertyHelper.js]
---*/


var obj = {};

function set_func(value) {
  obj.setVerifyHelpProp = value;
}

function get_func() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  get: get_func,
  set: set_func,
  enumerable: false,
  configurable: false
});

Object.defineProperties(obj, {
  foo: {
    get: get_func
  }
});
verifyEqualTo(obj, "foo", get_func());

verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
