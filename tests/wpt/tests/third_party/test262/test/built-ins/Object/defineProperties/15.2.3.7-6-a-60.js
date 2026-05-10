// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-60
description: >
    Object.defineProperties - desc.[[Set]] and P.[[Set]] are two
    objects which refer to the different objects (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

function set_func1() {}

Object.defineProperty(obj, "foo", {
  set: set_func1,
  configurable: true
});

function set_func2(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperties(obj, {
  foo: {
    set: set_func2
  }
});
verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: true,
});
