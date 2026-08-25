// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-109
description: >
    Object.defineProperties - 'P' is accessor property, several
    attributes values of P and properties are different (8.12.9 step
    12)
includes: [propertyHelper.js]
---*/


var obj = {};

function get_func1() {
  return 10;
}

function set_func1() {}

Object.defineProperty(obj, "foo", {
  get: get_func1,
  set: set_func1,
  configurable: true
});

function get_func2() {
  return 20;
}

function set_func2(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperties(obj, {
  foo: {
    get: get_func2,
    set: set_func2,
    configurable: false
  }
});
verifyEqualTo(obj, "foo", get_func2());

verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
