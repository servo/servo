// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-59
description: >
    Object.defineProperty - 'name' is accessor descriptor and every
    fields in 'desc' is absent (8.12.9 step 5)
includes: [propertyHelper.js]
---*/


var obj = {};

function getFunc() {
  return 0;
}

function setFunc(value) {
  obj.helpVerifySet = value;
}

Object.defineProperty(obj, "foo", {
  get: getFunc,
  set: setFunc
});

Object.defineProperty(obj, "foo", {});
verifyEqualTo(obj, "foo", getFunc());

verifyWritable(obj, "foo", "helpVerifySet");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
