// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-109
description: >
    Object.defineProperty - 'name' and 'desc' are accessor properties,
    name.[[Get]] is undefined and desc.[[Get]] is function (8.12.9
    step 12)
includes: [propertyHelper.js]
---*/

var obj = {};

function setFunc(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  set: setFunc,
  get: undefined,
  enumerable: true,
  configurable: true
});

function getFunc() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  get: getFunc
});
verifyEqualTo(obj, "foo", getFunc());

verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: true,
  configurable: true,
});
