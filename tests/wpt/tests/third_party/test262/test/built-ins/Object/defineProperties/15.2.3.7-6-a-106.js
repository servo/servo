// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-106
description: >
    Object.defineProperties - 'P' is accessor property, P.[[Set]] is
    undefined and properties.[[Set]] is normal value (8.12.9 step 12)
includes: [propertyHelper.js]
---*/


var obj = {};

function get_func() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  get: get_func,
  set: undefined,
  enumerable: true,
  configurable: true
});

function set_func(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperties(obj, {
  foo: {
    set: set_func
  }
});
verifyEqualTo(obj, "foo", get_func());

verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: true,
  configurable: true,
});
