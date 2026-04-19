// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-78
description: >
    Object.defineProperty - desc.[[Set]] and name.[[Set]] are two
    objects which refer to the different objects (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

function setFunc1() {}

Object.defineProperty(obj, "foo", {
  set: setFunc1,
  configurable: true
});

function setFunc2(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  set: setFunc2
});
verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: true,
});
