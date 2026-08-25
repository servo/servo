// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-96
description: >
    Object.defineProperty will not throw TypeError when
    name.configurable = false, both desc.[[Set]] and name.[[Set]] are
    two objects which refer to the same object (8.12.9 step 11.a.i)
includes: [propertyHelper.js]
---*/


var obj = {};

function setFunc(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  set: setFunc,
  configurable: false
});

Object.defineProperty(obj, "foo", {
  set: setFunc
});
verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
