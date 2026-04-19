// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-40
description: >
    Object.defineProperties - 'P' is accessor descriptor and every
    fields in 'desc' is the same with 'P' (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

function get_func() {
  return 0;
}

function set_func(value) {
  obj.setVerifyHelpProp = value;
}
var desc = {
  get: get_func,
  set: set_func,
  enumerable: true,
  configurable: true
};

Object.defineProperty(obj, "foo", desc);

Object.defineProperties(obj, {
  foo: {
    get: get_func,
    set: set_func,
    enumerable: true,
    configurable: true
  }
});
verifyEqualTo(obj, "foo", get_func());

verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: true,
  configurable: true,
});
