// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-86
description: >
    Object.defineProperties will not throw TypeError when
    P.configurable is false, both properties.[[Set]] and P.[[Set]] are
    two objects which refer to the same object (8.12.9 step 11.a.i)
includes: [propertyHelper.js]
---*/


var obj = {};

function set_func(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  set: set_func,
  configurable: false
});

Object.defineProperties(obj, {
  foo: {
    set: set_func
  }
});
verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});

