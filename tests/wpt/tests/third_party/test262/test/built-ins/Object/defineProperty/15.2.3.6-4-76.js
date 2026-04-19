// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-76
description: >
    Object.defineProperty - desc.[[Get]] and name.[[Get]] are two
    objects which refer to the different objects (8.12.9 step 6)
includes: [propertyHelper.js]
---*/


var obj = {};

function getFunc1() {
  return 10;
}

function setFunc1(value) {
  obj.helpVerifySet = value;
}

Object.defineProperty(obj, "foo", {
  get: getFunc1,
  set: setFunc1,
  configurable: true
});

function getFunc2() {
  return 20;
}

Object.defineProperty(obj, "foo", {
  get: getFunc2
});
verifyEqualTo(obj, "foo", getFunc2());

verifyWritable(obj, "foo", "helpVerifySet");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: true,
});
