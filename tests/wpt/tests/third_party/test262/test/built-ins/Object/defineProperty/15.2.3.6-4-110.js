// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-110
description: >
    Object.defineProperty - 'name' and 'desc' are accessor properties,
    both desc.[[Set]] and name.[[Set]] are two different values
    (8.12.9 step 12)
includes: [propertyHelper.js]
---*/


var obj = {};

function setFunc1() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  set: setFunc1,
  enumerable: true,
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
  enumerable: true,
  configurable: true,
});
