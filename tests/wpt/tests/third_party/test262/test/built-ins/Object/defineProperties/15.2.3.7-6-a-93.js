// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-93
description: >
    Object.defineProperties will not throw TypeError when
    P.configurable is false, P.[[Get]] and properties.[[Get]] are
    undefined (8.12.9 step 11.a.ii)
includes: [propertyHelper.js]
---*/


var obj = {};

function set_func(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  get: undefined,
  set: set_func,
  enumerable: false,
  configurable: false
});

Object.defineProperties(obj, {
  foo: {
    get: undefined
  }
});
verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
